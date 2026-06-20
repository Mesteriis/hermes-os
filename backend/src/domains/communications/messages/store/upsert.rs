use serde_json::json;

use super::MessageProjectionStore;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::ids::message_id;
use crate::domains::communications::messages::models::{NewProjectedMessage, ProjectedMessage};
use crate::domains::communications::messages::rows::row_to_projected_message;

impl MessageProjectionStore {
    pub async fn upsert_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        message.validate()?;
        let canonical_message_id = message_id(&message.account_id, &message.provider_record_id);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_messages (
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            )
            SELECT
                $1,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                $5,
                $6,
                $7,
                $8,
                $9,
                'email',
                NULL,
                $6,
                'received',
                '{}'::jsonb
            FROM communication_raw_records
            WHERE raw_record_id = $2
              AND account_id = $3
              AND provider_record_id = $4
              AND record_kind = 'email_message'
            ON CONFLICT (account_id, provider_record_id)
            DO UPDATE SET
                message_id = EXCLUDED.message_id,
                raw_record_id = EXCLUDED.raw_record_id,
                observation_id = EXCLUDED.observation_id,
                subject = EXCLUDED.subject,
                sender = EXCLUDED.sender,
                recipients = EXCLUDED.recipients,
                body_text = EXCLUDED.body_text,
                occurred_at = EXCLUDED.occurred_at,
                channel_kind = EXCLUDED.channel_kind,
                conversation_id = EXCLUDED.conversation_id,
                sender_display_name = EXCLUDED.sender_display_name,
                delivery_state = EXCLUDED.delivery_state,
                message_metadata = EXCLUDED.message_metadata,
                projected_at = now()
            RETURNING
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                workflow_state,
                importance_score,
                ai_category,
                ai_summary,
                ai_summary_generated_at,
                local_state,
                local_state_changed_at,
                local_state_reason
            "#,
        )
        .bind(&canonical_message_id)
        .bind(&message.raw_record_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.subject)
        .bind(&message.sender)
        .bind(json!(message.recipients))
        .bind(&message.body_text)
        .bind(message.occurred_at)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MessageProjectionError::RawRecordTupleMismatch {
                raw_record_id: message.raw_record_id.clone(),
                account_id: message.account_id.clone(),
                provider_record_id: message.provider_record_id.clone(),
            });
        };

        row_to_projected_message(row)
    }

    pub async fn upsert_channel_message(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.upsert_channel_message_with_body_policy(message, false)
            .await
    }

    pub async fn upsert_channel_message_allowing_empty_body_text(
        &self,
        message: &NewProjectedMessage,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        self.upsert_channel_message_with_body_policy(message, true)
            .await
    }

    async fn upsert_channel_message_with_body_policy(
        &self,
        message: &NewProjectedMessage,
        allow_empty_body_text: bool,
    ) -> Result<ProjectedMessage, MessageProjectionError> {
        message.validate_with_body_policy(allow_empty_body_text)?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_messages (
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            )
            SELECT
                $1,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                $12,
                $13,
                $14
            FROM communication_raw_records
            WHERE raw_record_id = $2
              AND account_id = $3
              AND provider_record_id = $4
            ON CONFLICT (account_id, provider_record_id)
            DO UPDATE SET
                message_id = EXCLUDED.message_id,
                raw_record_id = EXCLUDED.raw_record_id,
                observation_id = EXCLUDED.observation_id,
                subject = EXCLUDED.subject,
                sender = EXCLUDED.sender,
                recipients = EXCLUDED.recipients,
                body_text = EXCLUDED.body_text,
                occurred_at = EXCLUDED.occurred_at,
                channel_kind = EXCLUDED.channel_kind,
                conversation_id = EXCLUDED.conversation_id,
                sender_display_name = EXCLUDED.sender_display_name,
                delivery_state = EXCLUDED.delivery_state,
                message_metadata = EXCLUDED.message_metadata,
                projected_at = now()
            RETURNING
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata,
                workflow_state,
                importance_score,
                ai_category,
                ai_summary,
                ai_summary_generated_at,
                local_state,
                local_state_changed_at,
                local_state_reason
            "#,
        )
        .bind(&message.message_id)
        .bind(&message.raw_record_id)
        .bind(&message.account_id)
        .bind(&message.provider_record_id)
        .bind(&message.subject)
        .bind(&message.sender)
        .bind(json!(message.recipients))
        .bind(&message.body_text)
        .bind(message.occurred_at)
        .bind(&message.channel_kind)
        .bind(message.conversation_id.as_deref())
        .bind(message.sender_display_name.as_deref())
        .bind(&message.delivery_state)
        .bind(&message.message_metadata)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MessageProjectionError::RawRecordTupleMismatch {
                raw_record_id: message.raw_record_id.clone(),
                account_id: message.account_id.clone(),
                provider_record_id: message.provider_record_id.clone(),
            });
        };

        row_to_projected_message(row)
    }
}
