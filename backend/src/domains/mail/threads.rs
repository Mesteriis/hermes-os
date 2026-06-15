use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
#[allow(unused_imports)]
use sqlx::postgres::PgRow;
use thiserror::Error;

/// Normalize an email subject for thread grouping.
/// Strips Re:, Fwd:, AW:, WG: prefixes and whitespace.
pub fn normalize_subject_for_thread(subject: &str) -> String {
    let mut s = subject.trim().to_owned();
    loop {
        let lower = s.to_lowercase();
        let prefix_len = if lower.starts_with("re:") {
            "re:".len()
        } else if lower.starts_with("aw:") {
            "aw:".len()
        } else if lower.starts_with("wg:") {
            "wg:".len()
        } else if lower.starts_with("fwd:") {
            "fwd:".len()
        } else if lower.starts_with("fw:") {
            "fw:".len()
        } else {
            break;
        };
        s = s[prefix_len..].trim().to_owned();
    }
    s
}

/// Deterministic thread ID from account + normalized subject.
pub fn thread_id(account_id: &str, subject: &str) -> String {
    let normalized = normalize_subject_for_thread(subject);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&account_id, &mut hasher);
    std::hash::Hash::hash(&normalized.to_lowercase(), &mut hasher);
    format!("thread:{:016x}", std::hash::Hasher::finish(&hasher))
}

#[derive(Clone, Debug, Serialize)]
pub struct EmailThread {
    pub thread_id: String,
    pub account_id: String,
    pub subject: String,
    pub message_count: i64,
    pub participant_count: i64,
    pub first_message_at: Option<DateTime<Utc>>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub last_activity_at: DateTime<Utc>,
    pub has_open_action: bool,
    pub has_attachments: bool,
    pub dominant_workflow_state: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct EmailThreadListPage {
    pub items: Vec<EmailThread>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone)]
pub struct EmailThreadStore {
    pool: PgPool,
}

impl EmailThreadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List threads for an account, ordered by most recent activity.
    pub async fn list_threads(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<EmailThread>, EmailThreadError> {
        Ok(self.list_threads_page(account_id, None, limit).await?.items)
    }

    pub async fn list_threads_page(
        &self,
        account_id: Option<&str>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<EmailThreadListPage, EmailThreadError> {
        let limit = if (1..=100).contains(&limit) {
            limit
        } else {
            50
        };
        let cursor = cursor.map(decode_thread_list_cursor).transpose()?;

        let rows = sqlx::query(
            r#"
            WITH grouped_threads AS (
                SELECT
                    COALESCE(m.conversation_id, md5(m.account_id || ':' || lower(regexp_replace(regexp_replace(regexp_replace(m.subject, '^re:\s*', '', 'i'), '^fwd:\s*', '', 'i'), '^aw:\s*', '', 'i')))) AS thread_id,
                    m.account_id,
                    regexp_replace(regexp_replace(regexp_replace(m.subject, '^re:\s*', '', 'i'), '^fwd:\s*', '', 'i'), '^aw:\s*', '', 'i') AS normalized_subject,
                    count(*)::BIGINT AS message_count,
                    count(DISTINCT m.sender)::BIGINT AS participant_count,
                    min(m.occurred_at) AS first_message_at,
                    max(m.occurred_at) AS last_message_at,
                    max(COALESCE(m.occurred_at, m.projected_at)) AS last_activity_at,
                    bool_or(m.workflow_state IN ('needs_action', 'new')) AS has_open_action,
                    bool_or(EXISTS(SELECT 1 FROM communication_attachments a WHERE a.message_id = m.message_id)) AS has_attachments,
                    mode() WITHIN GROUP (ORDER BY m.workflow_state) AS dominant_workflow_state
                FROM communication_messages m
                WHERE ($1::text IS NULL OR m.account_id = $1)
                  AND m.channel_kind = 'email'
                  AND m.local_state = 'active'
                GROUP BY thread_id, m.account_id, normalized_subject
            )
            SELECT
                thread_id,
                account_id,
                normalized_subject,
                message_count,
                participant_count,
                first_message_at,
                last_message_at,
                last_activity_at,
                has_open_action,
                has_attachments,
                dominant_workflow_state
            FROM grouped_threads
            WHERE (
                $2::timestamptz IS NULL
                OR last_activity_at < $2
                OR (last_activity_at = $2 AND thread_id > $3)
            )
            ORDER BY last_activity_at DESC, thread_id ASC
            LIMIT $4
            "#,
        )
        .bind(account_id)
        .bind(cursor.as_ref().map(|cursor| cursor.last_activity_at))
        .bind(cursor.as_ref().map(|cursor| cursor.thread_id.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;

        let mut threads = Vec::new();
        for row in rows {
            threads.push(EmailThread {
                thread_id: row.try_get("thread_id")?,
                account_id: row.try_get("account_id")?,
                subject: row.try_get("normalized_subject")?,
                message_count: row.try_get("message_count")?,
                participant_count: row.try_get("participant_count")?,
                first_message_at: row.try_get("first_message_at")?,
                last_message_at: row.try_get("last_message_at")?,
                last_activity_at: row.try_get("last_activity_at")?,
                has_open_action: row.try_get("has_open_action")?,
                has_attachments: row.try_get("has_attachments")?,
                dominant_workflow_state: row.try_get::<String, _>("dominant_workflow_state")?,
            });
        }

        let has_more = threads.len() > limit as usize;
        if has_more {
            threads.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            threads.last().map(encode_thread_list_cursor).transpose()?
        } else {
            None
        };

        Ok(EmailThreadListPage {
            items: threads,
            next_cursor,
            has_more,
        })
    }

    /// Get messages belonging to a thread, identified by normalized subject + account_id.
    pub async fn thread_messages(
        &self,
        account_id: &str,
        normalized_subject: &str,
        limit: i64,
    ) -> Result<Vec<ThreadMessage>, EmailThreadError> {
        let limit = if (1..=100).contains(&limit) {
            limit
        } else {
            50
        };

        let rows = sqlx::query(
            r#"
            SELECT
                m.message_id,
                m.provider_record_id,
                m.account_id,
                m.subject,
                m.sender,
                m.sender_display_name,
                m.body_text,
                m.occurred_at,
                m.projected_at,
                m.workflow_state,
                m.importance_score,
                m.ai_category,
                m.ai_summary,
                m.delivery_state,
                count(a.attachment_id)::BIGINT AS attachment_count,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'attachment_id', a.attachment_id,
                            'message_id', a.message_id,
                            'raw_record_id', a.raw_record_id,
                            'blob_id', a.blob_id,
                            'provider_attachment_id', a.provider_attachment_id,
                            'filename', a.filename,
                            'content_type', a.content_type,
                            'size_bytes', a.size_bytes,
                            'sha256', a.sha256,
                            'disposition', a.disposition,
                            'scan_status', a.scan_status,
                            'scan_engine', a.scan_engine,
                            'scan_checked_at', a.scan_checked_at,
                            'scan_summary', a.scan_summary,
                            'scan_metadata', a.scan_metadata,
                            'storage_kind', a.storage_kind,
                            'storage_path', a.storage_path,
                            'created_at', a.created_at,
                            'updated_at', a.updated_at
                        )
                        ORDER BY a.created_at ASC
                    ) FILTER (WHERE a.attachment_id IS NOT NULL),
                    '[]'::jsonb
                ) AS attachments
            FROM communication_messages m
            LEFT JOIN communication_attachments a ON a.message_id = m.message_id
            WHERE m.account_id = $1
              AND m.channel_kind = 'email'
              AND m.local_state = 'active'
              AND regexp_replace(regexp_replace(regexp_replace(m.subject, '^re:\s*', '', 'i'), '^fwd:\s*', '', 'i'), '^aw:\s*', '', 'i') = $2
            GROUP BY m.message_id, m.provider_record_id, m.account_id, m.subject, m.sender, m.sender_display_name,
                     m.body_text, m.occurred_at, m.projected_at, m.workflow_state,
                     m.importance_score, m.ai_category, m.ai_summary, m.delivery_state
            ORDER BY COALESCE(m.occurred_at, m.projected_at) ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(normalized_subject)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(ThreadMessage {
                message_id: row.try_get("message_id")?,
                provider_record_id: row.try_get("provider_record_id")?,
                account_id: row.try_get("account_id")?,
                subject: row.try_get("subject")?,
                sender: row.try_get("sender")?,
                sender_display_name: row.try_get("sender_display_name")?,
                body_text: row.try_get("body_text")?,
                occurred_at: row.try_get("occurred_at")?,
                projected_at: row.try_get("projected_at")?,
                workflow_state: row.try_get("workflow_state")?,
                importance_score: row.try_get("importance_score")?,
                ai_category: row.try_get("ai_category")?,
                ai_summary: row.try_get("ai_summary")?,
                delivery_state: row.try_get("delivery_state")?,
                attachment_count: row.try_get("attachment_count")?,
                attachments: serde_json::from_value(row.try_get::<Value, _>("attachments")?)
                    .map_err(EmailThreadError::Serde)?,
            });
        }

        Ok(messages)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ThreadMessage {
    pub message_id: String,
    pub provider_record_id: String,
    pub account_id: String,
    pub subject: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub workflow_state: String,
    pub importance_score: Option<i16>,
    pub ai_category: Option<String>,
    pub ai_summary: Option<String>,
    pub delivery_state: String,
    pub attachment_count: i64,
    pub attachments: Vec<ThreadMessageAttachment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadMessageAttachment {
    pub attachment_id: String,
    pub message_id: String,
    pub raw_record_id: String,
    pub blob_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub disposition: String,
    pub scan_status: String,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
    pub scan_metadata: Value,
    pub storage_kind: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ThreadListCursor {
    last_activity_at: DateTime<Utc>,
    thread_id: String,
}

fn encode_thread_list_cursor(thread: &EmailThread) -> Result<String, EmailThreadError> {
    let cursor = ThreadListCursor {
        last_activity_at: thread.last_activity_at,
        thread_id: thread.thread_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| EmailThreadError::InvalidCursor)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_thread_list_cursor(cursor: &str) -> Result<ThreadListCursor, EmailThreadError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| EmailThreadError::InvalidCursor)?;
    let cursor: ThreadListCursor =
        serde_json::from_slice(&bytes).map_err(|_| EmailThreadError::InvalidCursor)?;
    if cursor.thread_id.trim().is_empty() {
        return Err(EmailThreadError::InvalidCursor);
    }

    Ok(cursor)
}

#[derive(Debug, Error)]
pub enum EmailThreadError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("invalid thread cursor")]
    InvalidCursor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_subject_strips_re_prefix() {
        assert_eq!(
            normalize_subject_for_thread("Re: Hello World"),
            "Hello World"
        );
        assert_eq!(normalize_subject_for_thread("RE: Hello"), "Hello");
        assert_eq!(normalize_subject_for_thread("Re: Re: FWD: Hello"), "Hello");
    }

    #[test]
    fn normalize_subject_strips_fwd_prefix() {
        assert_eq!(normalize_subject_for_thread("Fwd: Important"), "Important");
        assert_eq!(normalize_subject_for_thread("FW: Document"), "Document");
    }

    #[test]
    fn normalize_subject_strips_aw_prefix() {
        assert_eq!(
            normalize_subject_for_thread("AW: Meeting Notes"),
            "Meeting Notes"
        );
        assert_eq!(normalize_subject_for_thread("WG: Conference"), "Conference");
    }

    #[test]
    fn normalize_subject_handles_whitespace() {
        assert_eq!(normalize_subject_for_thread("  Re:   Spaced  "), "Spaced");
    }

    #[test]
    fn thread_id_is_deterministic() {
        let id1 = thread_id("acct1", "Hello World");
        let id2 = thread_id("acct1", "Re: Hello World");
        let id3 = thread_id("acct1", "FWD: Hello World");
        assert_eq!(id1, id2);
        assert_eq!(id1, id3);
    }

    #[test]
    fn thread_id_different_for_different_accounts() {
        let id1 = thread_id("acct1", "Hello");
        let id2 = thread_id("acct2", "Hello");
        assert_ne!(id1, id2);
    }
}
