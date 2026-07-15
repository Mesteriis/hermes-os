use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct MailboxHealth {
    pub total_messages: i64,
    pub unread: i64,
    pub needs_action: i64,
    pub waiting: i64,
    pub done: i64,
    pub archived: i64,
    pub spam: i64,
    pub important: i64,
    pub with_attachments: i64,
    pub average_importance: f64,
    pub oldest_message_days: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SenderStats {
    pub sender: String,
    pub message_count: i64,
    pub avg_importance: f64,
    pub last_message_days: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SenderStatsListPage {
    pub items: Vec<SenderStats>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone)]
pub struct EmailAnalyticsStore {
    pool: PgPool,
}

impl EmailAnalyticsStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn mailbox_health(
        &self,
        account_id: Option<&str>,
    ) -> Result<MailboxHealth, EmailAnalyticsError> {
        let row = sqlx::query(
            r#"SELECT
                count(*)::BIGINT AS total_messages,
                count(*) FILTER (WHERE workflow_state = 'new')::BIGINT AS unread,
                count(*) FILTER (WHERE workflow_state = 'needs_action')::BIGINT AS needs_action,
                count(*) FILTER (WHERE workflow_state = 'waiting')::BIGINT AS waiting,
                count(*) FILTER (WHERE workflow_state = 'done')::BIGINT AS done,
                count(*) FILTER (WHERE workflow_state = 'archived')::BIGINT AS archived,
                count(*) FILTER (WHERE workflow_state = 'spam')::BIGINT AS spam,
                count(*) FILTER (WHERE importance_score >= 75)::BIGINT AS important,
                count(*) FILTER (WHERE EXISTS(SELECT 1 FROM communication_attachments a WHERE a.message_id = communication_messages.message_id))::BIGINT AS with_attachments,
                COALESCE(avg(importance_score), 0)::DOUBLE PRECISION AS average_importance,
                EXTRACT(EPOCH FROM now() - min(occurred_at))::DOUBLE PRECISION / 86400.0::DOUBLE PRECISION AS oldest_message_days
            FROM communication_messages
            WHERE ($1::text IS NULL OR account_id = $1)
              AND channel_kind = 'email'
              AND local_state = 'active'"#,
        ).bind(account_id).fetch_one(&self.pool).await?;

        Ok(MailboxHealth {
            total_messages: row.try_get("total_messages")?,
            unread: row.try_get("unread")?,
            needs_action: row.try_get("needs_action")?,
            waiting: row.try_get("waiting")?,
            done: row.try_get("done")?,
            archived: row.try_get("archived")?,
            spam: row.try_get("spam")?,
            important: row.try_get("important")?,
            with_attachments: row.try_get("with_attachments")?,
            average_importance: row.try_get("average_importance")?,
            oldest_message_days: row.try_get("oldest_message_days")?,
        })
    }

    pub async fn top_senders(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SenderStats>, EmailAnalyticsError> {
        Ok(self.top_senders_page(account_id, limit, None).await?.items)
    }

    pub async fn top_senders_page(
        &self,
        account_id: Option<&str>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<SenderStatsListPage, EmailAnalyticsError> {
        let limit = limit.clamp(1, 50);
        let cursor = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(decode_sender_stats_cursor)
            .transpose()?;
        let rows = sqlx::query(
            r#"WITH sender_stats AS (
                SELECT sender, count(*)::BIGINT AS message_count,
                    COALESCE(avg(importance_score), 0)::DOUBLE PRECISION AS avg_importance,
                    EXTRACT(EPOCH FROM now() - max(occurred_at))::DOUBLE PRECISION / 86400.0::DOUBLE PRECISION AS last_message_days
                FROM communication_messages
                WHERE ($1::text IS NULL OR account_id = $1)
                  AND channel_kind = 'email'
                  AND local_state = 'active'
                GROUP BY sender
            )
            SELECT sender, message_count, avg_importance, last_message_days
            FROM sender_stats
            WHERE (
                $2::BIGINT IS NULL
                OR message_count < $2
                OR (message_count = $2 AND sender > $3)
            )
            ORDER BY message_count DESC, sender ASC
            LIMIT $4"#,
        )
        .bind(account_id)
        .bind(cursor.as_ref().map(|value| value.message_count))
        .bind(cursor.as_ref().map(|value| value.sender.as_str()))
        .bind(limit + 1)
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(SenderStats {
                sender: row.try_get("sender")?,
                message_count: row.try_get("message_count")?,
                avg_importance: row.try_get("avg_importance")?,
                last_message_days: row.try_get("last_message_days")?,
            });
        }
        let has_more = stats.len() > limit as usize;
        if has_more {
            stats.truncate(limit as usize);
        }
        let next_cursor = if has_more {
            stats.last().map(encode_sender_stats_cursor).transpose()?
        } else {
            None
        };
        Ok(SenderStatsListPage {
            items: stats,
            next_cursor,
            has_more,
        })
    }
}

#[derive(Debug, Error)]
pub enum EmailAnalyticsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid sender stats cursor")]
    InvalidCursor,
}

#[derive(Debug, Deserialize, Serialize)]
struct SenderStatsCursor {
    message_count: i64,
    sender: String,
}

fn encode_sender_stats_cursor(sender: &SenderStats) -> Result<String, EmailAnalyticsError> {
    let cursor = SenderStatsCursor {
        message_count: sender.message_count,
        sender: sender.sender.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| EmailAnalyticsError::InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_sender_stats_cursor(cursor: &str) -> Result<SenderStatsCursor, EmailAnalyticsError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| EmailAnalyticsError::InvalidCursor)?;
    let cursor: SenderStatsCursor =
        serde_json::from_slice(&bytes).map_err(|_| EmailAnalyticsError::InvalidCursor)?;
    if cursor.message_count < 0 || cursor.sender.trim().is_empty() {
        return Err(EmailAnalyticsError::InvalidCursor);
    }
    Ok(cursor)
}
