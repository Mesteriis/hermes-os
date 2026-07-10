use serde_json::Value;

use super::MessageProjectionStore;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::models::ProjectedMessage;
use crate::domains::communications::messages::rows::row_to_projected_message;
use crate::domains::communications::messages::validation::validate_non_empty;

impl MessageProjectionStore {
    pub async fn set_ai_analysis(
        &self,
        message_id: &str,
        category: Option<&str>,
        summary: Option<&str>,
        importance_score: Option<i16>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        if let Some(score) = importance_score
            && !(0..=100).contains(&score)
        {
            return Err(MessageProjectionError::InvalidImportanceScore(score));
        }
        let row = sqlx::query(
            r#"UPDATE communication_messages SET
                ai_category = COALESCE($2, ai_category),
                ai_summary = COALESCE($3, ai_summary),
                ai_summary_generated_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE ai_summary_generated_at END,
                importance_score = COALESCE($4, importance_score),
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
        .bind(category)
        .bind(summary)
        .bind(importance_score)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Err(MessageProjectionError::MessageNotFound);
        };
        row_to_projected_message(row)
    }

    pub async fn set_message_metadata(
        &self,
        message_id: &str,
        metadata: &Value,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.set_message_metadata_with_observation(
            message_id,
            metadata,
            None,
            "message_flag_update",
            None,
        )
        .await
    }

    pub async fn set_message_metadata_with_observation(
        &self,
        message_id: &str,
        metadata: &Value,
        observation_id: Option<&str>,
        relationship_kind: &str,
        link_metadata: Option<Value>,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        validate_non_empty("message_id", message_id)?;
        if !metadata.is_object() {
            return Err(MessageProjectionError::InvalidMessageMetadata);
        }
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"UPDATE communication_messages SET message_metadata = $2, projected_at = now()
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
        .bind(metadata)
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
                serde_json::json!({}),
                link_metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(message)
    }
}
