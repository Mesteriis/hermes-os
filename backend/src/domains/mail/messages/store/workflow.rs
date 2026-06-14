use sqlx::{Postgres, Transaction};

use super::MessageProjectionStore;
use crate::domains::mail::messages::errors::MessageProjectionError;
use crate::domains::mail::messages::models::ProjectedMessage;
use crate::domains::mail::messages::rows::row_to_projected_message;
use crate::domains::mail::messages::states::WorkflowState;
use crate::domains::mail::messages::validation::validate_non_empty;

impl MessageProjectionStore {
    pub async fn transition_workflow_state(
        &self,
        message_id: &str,
        new_state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let message =
            Self::transition_workflow_state_in_transaction(&mut transaction, message_id, new_state)
                .await?;
        transaction.commit().await?;
        Ok(message)
    }

    pub(crate) async fn transition_workflow_state_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        message_id: &str,
        new_state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        let row = sqlx::query(
            r#"UPDATE communication_messages SET workflow_state = $2, projected_at = now()
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
        .bind(new_state.as_str())
        .fetch_optional(&mut **transaction)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }
}
