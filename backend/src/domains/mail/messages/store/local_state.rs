use super::MessageProjectionStore;
use crate::domains::mail::messages::errors::MessageProjectionError;
use crate::domains::mail::messages::models::ProjectedMessage;
use crate::domains::mail::messages::rows::row_to_projected_message;
use crate::domains::mail::messages::validation::validate_non_empty;

impl MessageProjectionStore {
    pub async fn move_to_local_trash(
        &self,
        message_id: &str,
        reason: &str,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        validate_non_empty("local_state_reason", reason)?;
        let row = sqlx::query(
            r#"UPDATE communication_messages
            SET local_state = 'trash',
                local_state_changed_at = now(),
                local_state_reason = $2,
                projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                local_state, local_state_changed_at, local_state_reason"#,
        )
        .bind(message_id.trim())
        .bind(reason.trim())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }

    pub async fn restore_from_local_trash(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        let row = sqlx::query(
            r#"UPDATE communication_messages
            SET local_state = 'active',
                local_state_changed_at = now(),
                local_state_reason = NULL,
                projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                local_state, local_state_changed_at, local_state_reason"#,
        )
        .bind(message_id.trim())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }
}
