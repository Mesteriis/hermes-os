use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailDraft {
    pub draft_id: String,
    pub account_id: String,
    pub persona_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub status: DraftStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub send_attempts: i32,
    pub last_error: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Failed,
}

impl DraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DraftStatus::Draft => "draft",
            DraftStatus::Scheduled => "scheduled",
            DraftStatus::Sending => "sending",
            DraftStatus::Sent => "sent",
            DraftStatus::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "draft" => Some(DraftStatus::Draft),
            "scheduled" => Some(DraftStatus::Scheduled),
            "sending" => Some(DraftStatus::Sending),
            "sent" => Some(DraftStatus::Sent),
            "failed" => Some(DraftStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewEmailDraft {
    pub draft_id: String,
    pub account_id: String,
    pub persona_id: Option<String>,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub status: DraftStatus,
    pub scheduled_send_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewEmailDraft {
    fn validate(&self) -> Result<(), EmailDraftError> {
        if self.draft_id.trim().is_empty() {
            return Err(EmailDraftError::Invalid("draft_id empty"));
        }
        if self.account_id.trim().is_empty() {
            return Err(EmailDraftError::Invalid("account_id empty"));
        }
        if self.subject.trim().is_empty() {
            return Err(EmailDraftError::Invalid("subject empty"));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct EmailDraftStore {
    pool: PgPool,
}

impl EmailDraftStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, draft: &NewEmailDraft) -> Result<EmailDraft, EmailDraftError> {
        draft.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO email_drafts (draft_id, account_id, persona_id, to_recipients, cc_recipients, bcc_recipients, subject, body_text, body_html, in_reply_to, message_references, status, scheduled_send_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (draft_id) DO UPDATE SET
                account_id = EXCLUDED.account_id, persona_id = EXCLUDED.persona_id,
                to_recipients = EXCLUDED.to_recipients, cc_recipients = EXCLUDED.cc_recipients,
                bcc_recipients = EXCLUDED.bcc_recipients, subject = EXCLUDED.subject,
                body_text = EXCLUDED.body_text, body_html = EXCLUDED.body_html,
                in_reply_to = EXCLUDED.in_reply_to, message_references = EXCLUDED.message_references,
                status = EXCLUDED.status, scheduled_send_at = EXCLUDED.scheduled_send_at,
                metadata = EXCLUDED.metadata, updated_at = now()
            RETURNING draft_id, account_id, persona_id, to_recipients, cc_recipients, bcc_recipients, subject, body_text, body_html, in_reply_to, message_references, status, scheduled_send_at, send_attempts, last_error, metadata, created_at, updated_at"#,
        )
        .bind(&draft.draft_id).bind(&draft.account_id).bind(draft.persona_id.as_deref())
        .bind(serde_json::to_value(&draft.to_recipients).unwrap_or_default())
        .bind(serde_json::to_value(&draft.cc_recipients).unwrap_or_default())
        .bind(serde_json::to_value(&draft.bcc_recipients).unwrap_or_default())
        .bind(&draft.subject).bind(&draft.body_text).bind(draft.body_html.as_deref())
        .bind(draft.in_reply_to.as_deref())
        .bind(serde_json::to_value(&draft.references).unwrap_or_default())
        .bind(draft.status.as_str()).bind(draft.scheduled_send_at).bind(&draft.metadata)
        .fetch_one(&self.pool).await?;
        row_to_draft(row)
    }

    pub async fn list(
        &self,
        account_id: Option<&str>,
        status: Option<DraftStatus>,
    ) -> Result<Vec<EmailDraft>, EmailDraftError> {
        let status_str = status.map(|s| s.as_str().to_owned());
        let rows = sqlx::query(
            r#"SELECT draft_id, account_id, persona_id, to_recipients, cc_recipients, bcc_recipients, subject, body_text, body_html, in_reply_to, message_references, status, scheduled_send_at, send_attempts, last_error, metadata, created_at, updated_at
            FROM email_drafts WHERE ($1::text IS NULL OR account_id = $1) AND ($2::text IS NULL OR status = $2) ORDER BY updated_at DESC"#,
        )
        .bind(account_id).bind(status_str.as_deref())
        .fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_draft).collect()
    }

    pub async fn get(&self, draft_id: &str) -> Result<Option<EmailDraft>, EmailDraftError> {
        let row = sqlx::query(
            r#"SELECT draft_id, account_id, persona_id, to_recipients, cc_recipients, bcc_recipients, subject, body_text, body_html, in_reply_to, message_references, status, scheduled_send_at, send_attempts, last_error, metadata, created_at, updated_at
            FROM email_drafts WHERE draft_id = $1"#,
        ).bind(draft_id).fetch_optional(&self.pool).await?;
        row.map(row_to_draft).transpose()
    }

    pub async fn delete(&self, draft_id: &str) -> Result<bool, EmailDraftError> {
        let result = sqlx::query("DELETE FROM email_drafts WHERE draft_id = $1")
            .bind(draft_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn mark_stale_drafts(&self, older_than_days: i32) -> Result<u64, EmailDraftError> {
        let result = sqlx::query(
            "UPDATE email_drafts SET status = 'failed', last_error = 'stale draft expired', updated_at = now() WHERE status = 'draft' AND updated_at < now() - ($1 || ' days')::interval",
        ).bind(older_than_days).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }
}

fn row_to_draft(row: PgRow) -> Result<EmailDraft, EmailDraftError> {
    let status_str: String = row.try_get("status")?;
    Ok(EmailDraft {
        draft_id: row.try_get("draft_id")?,
        account_id: row.try_get("account_id")?,
        persona_id: row.try_get("persona_id")?,
        to_recipients: serde_json::from_value(row.try_get("to_recipients")?).unwrap_or_default(),
        cc_recipients: serde_json::from_value(row.try_get("cc_recipients")?).unwrap_or_default(),
        bcc_recipients: serde_json::from_value(row.try_get("bcc_recipients")?).unwrap_or_default(),
        subject: row.try_get("subject")?,
        body_text: row.try_get("body_text")?,
        body_html: row.try_get("body_html")?,
        in_reply_to: row.try_get("in_reply_to")?,
        references: serde_json::from_value(row.try_get("message_references")?).unwrap_or_default(),
        status: DraftStatus::parse(&status_str).unwrap_or(DraftStatus::Draft),
        scheduled_send_at: row.try_get("scheduled_send_at")?,
        send_attempts: row.try_get("send_attempts")?,
        last_error: row.try_get("last_error")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum EmailDraftError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid draft: {0}")]
    Invalid(&'static str),
}
