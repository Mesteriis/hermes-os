use super::MessageProjectionStore;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::rows::row_to_projected_message;
use crate::domains::communications::messages::validation::validate_non_empty;

impl MessageProjectionStore {
    pub async fn move_to_local_trash(
        &self,
        message_id: &str,
        reason: &str,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.move_to_local_trash_with_observation(
            message_id,
            reason,
            None,
            "local_state_transition",
            None,
        )
        .await
    }

    pub async fn move_to_local_trash_with_observation(
        &self,
        message_id: &str,
        reason: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        validate_non_empty("local_state_reason", reason)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"UPDATE communication_messages
            SET local_state = 'trash',
                local_state_changed_at = now(),
                local_state_reason = $2,
                projected_at = now()
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
        .bind(reason.trim())
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        let message = row_to_projected_message(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                message.message_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "local_state": message.local_state.as_str(),
                    "source": reason,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(message)
    }

    pub async fn restore_from_local_trash(
        &self,
        message_id: &str,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.restore_from_local_trash_with_observation(
            message_id,
            None,
            "local_state_transition",
            None,
        )
        .await
    }

    pub async fn restore_from_local_trash_with_observation(
        &self,
        message_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"UPDATE communication_messages
            SET local_state = 'active',
                local_state_changed_at = now(),
                local_state_reason = NULL,
                projected_at = now()
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
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        let message = row_to_projected_message(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "communication_message",
                message.message_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "local_state": message.local_state.as_str(),
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(message)
    }
}
