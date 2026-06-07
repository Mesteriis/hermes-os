use crate::domains::mail::messages::{MessageProjectionError, MessageProjectionStore};
use crate::domains::mail::storage::{MailStorageError, MailStorageStore};

#[derive(Debug, Clone)]
pub struct EmailExport {
    pub format: ExportFormat,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Eml,
    Markdown,
    Json,
}

impl ExportFormat {
    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Eml => "message/rfc822",
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Json => "application/json",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Eml => "eml",
            ExportFormat::Markdown => "md",
            ExportFormat::Json => "json",
        }
    }
}

pub async fn export_message(
    message_store: &MessageProjectionStore,
    attachment_store: &MailStorageStore,
    message_id: &str,
    format: ExportFormat,
) -> Result<EmailExport, EmailExportError> {
    let msg = message_store
        .message(message_id)
        .await?
        .ok_or(EmailExportError::NotFound)?;
    let attachments = attachment_store.attachments_for_message(message_id).await?;

    let content = match format {
        ExportFormat::Markdown => format!(
            "# {}\n\n**From:** {}\n**To:** {}\n**Date:** {}\n**State:** {}\n\n{}\n\n---\n*{} attachment(s)*",
            msg.subject,
            msg.sender,
            msg.recipients.join(", "),
            msg.occurred_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            msg.workflow_state.as_str(),
            msg.body_text,
            attachments.len(),
        ),
        ExportFormat::Eml => format!(
            "From: {}\r\nTo: {}\r\nSubject: {}\r\nDate: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
            msg.sender,
            msg.recipients.join(", "),
            msg.subject,
            msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
            msg.body_text,
        ),
        ExportFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "message_id": msg.message_id,
            "subject": msg.subject,
            "sender": msg.sender,
            "recipients": msg.recipients,
            "body_text": msg.body_text,
            "occurred_at": msg.occurred_at,
            "workflow_state": msg.workflow_state.as_str(),
            "importance_score": msg.importance_score,
            "ai_category": msg.ai_category,
            "ai_summary": msg.ai_summary,
            "attachment_count": attachments.len(),
        }))
        .unwrap_or_default(),
    };

    Ok(EmailExport { format, content })
}

#[derive(Debug, thiserror::Error)]
pub enum EmailExportError {
    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),
    #[error(transparent)]
    MailStorage(#[from] MailStorageError),
    #[error("message not found")]
    NotFound,
}
