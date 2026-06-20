use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::MessageProjectionError;
use super::models::{ProjectedMessage, ProjectedMessageSummary};
use super::payload::recipients_from_value;
use super::states::{LocalMessageState, WorkflowState};

pub(crate) fn row_to_projected_message_summary(
    row: PgRow,
) -> Result<ProjectedMessageSummary, MessageProjectionError> {
    let attachment_count = row.try_get("attachment_count")?;
    Ok(ProjectedMessageSummary {
        message: row_to_projected_message(row)?,
        attachment_count,
    })
}

pub(crate) fn row_to_projected_message(
    row: PgRow,
) -> Result<ProjectedMessage, MessageProjectionError> {
    let workflow_state: String = row.try_get("workflow_state")?;
    let local_state: String = row.try_get("local_state")?;
    Ok(ProjectedMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        observation_id: row.try_get("observation_id")?,
        account_id: row.try_get("account_id")?,
        provider_record_id: row.try_get("provider_record_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        recipients: recipients_from_value(row.try_get("recipients")?)?,
        body_text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        conversation_id: row.try_get("conversation_id")?,
        sender_display_name: row.try_get("sender_display_name")?,
        delivery_state: row.try_get("delivery_state")?,
        message_metadata: row.try_get("message_metadata")?,
        workflow_state: workflow_state
            .parse::<WorkflowState>()
            .unwrap_or(WorkflowState::New),
        importance_score: row.try_get("importance_score")?,
        ai_category: row.try_get("ai_category")?,
        ai_summary: row.try_get("ai_summary")?,
        ai_summary_generated_at: row.try_get("ai_summary_generated_at")?,
        local_state: local_state
            .parse::<LocalMessageState>()
            .unwrap_or(LocalMessageState::Active),
        local_state_changed_at: row.try_get("local_state_changed_at")?,
        local_state_reason: row.try_get("local_state_reason")?,
    })
}
