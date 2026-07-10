use sqlx::{Postgres, Transaction};

use super::MessageProjectionStore;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::rows::row_to_projected_message;
use crate::domains::communications::messages::states::WorkflowState;
use crate::domains::communications::messages::validation::validate_non_empty;

impl MessageProjectionStore {
    pub async fn transition_workflow_state(
        &self,
        message_id: &str,
        new_state: WorkflowState,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.transition_workflow_state_with_observation(
            message_id,
            new_state,
            None,
            "workflow_state_transition",
            None,
        )
        .await
    }

    pub async fn transition_workflow_state_with_observation(
        &self,
        message_id: &str,
        new_state: WorkflowState,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let message =
            Self::transition_workflow_state_in_transaction(&mut transaction, message_id, new_state)
                .await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                message.message_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "workflow_state": message.workflow_state.as_str(),
                }),
                metadata,
            )
            .await?;
        }
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
                message_id, raw_record_id, observation_id, account_id, provider_record_id,
                subject, sender, recipients, body_text,
                occurred_at, projected_at, channel_kind, conversation_id,
                sender_display_name, delivery_state, message_metadata,
                workflow_state, importance_score, ai_category,
                ai_summary, ai_summary_generated_at,
                (SELECT s.ai_state FROM communication_ai_states s WHERE s.message_id = communication_messages.message_id) AS ai_state,
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
