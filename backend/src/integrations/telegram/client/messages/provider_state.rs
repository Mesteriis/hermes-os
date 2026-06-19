use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{Postgres, Row, Transaction};

use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

use super::super::errors::TelegramError;
use super::super::evidence::link_communication_entity_in_transaction;
use super::super::models::TelegramMessage;
use super::super::rows::row_to_telegram_message;
use super::super::store::TelegramStore;

pub(super) async fn capture_message_projection_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    kind_code: &str,
    message: &TelegramMessage,
    observed_at: DateTime<Utc>,
    relationship_kind: &str,
    payload: Value,
    actor: &str,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            kind_code,
            ObservationOriginKind::LocalRuntime,
            observed_at,
            payload,
            format!("message://{}/{}", message.message_id, relationship_kind),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
            "account_id": message.account_id,
            "provider_message_id": message.provider_message_id,
            "provider_chat_id": message.provider_chat_id,
        })),
    )
    .await?;
    link_communication_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "communication_message",
        message.message_id.clone(),
        relationship_kind,
        json!({
            "account_id": message.account_id,
            "provider_message_id": message.provider_message_id,
            "provider_chat_id": message.provider_chat_id,
        }),
    )
    .await?;
    Ok(())
}

impl TelegramStore {
    pub(in crate::integrations::telegram) async fn message_by_provider_message_id(
        &self,
        account_id: &str,
        provider_message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE account_id = $1
              AND provider_record_id = $2
              AND channel_kind IN ('telegram_user', 'telegram_bot')
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_message_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_message).transpose()
    }

    pub(in crate::integrations::telegram) async fn apply_message_metadata(
        &self,
        message_id: &str,
        metadata: &Value,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        if !metadata.is_object() {
            return Err(TelegramError::InvalidRequest(
                "telegram message metadata must be a JSON object".to_owned(),
            ));
        }
        let Some(current) = self.message_by_id(message_id).await? else {
            return Ok(None);
        };

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = $2,
                projected_at = now()
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(message_id.trim())
        .bind(metadata)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_telegram_message(row)?;
        capture_message_projection_observation_in_transaction(
            &mut transaction,
            "COMMUNICATION_MESSAGE",
            &updated,
            updated.projected_at,
            "telegram_metadata_update",
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_message_id,
                "provider_chat_id": updated.provider_chat_id,
                "previous_metadata": current.metadata,
                "message_metadata": updated.metadata,
            }),
            "telegram.client.messages.provider_state.apply_message_metadata",
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }

    pub(in crate::integrations::telegram) async fn set_message_delivery_state(
        &self,
        message_id: &str,
        delivery_state: &str,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        let Some(current) = self.message_by_id(message_id).await? else {
            return Ok(None);
        };

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET delivery_state = $2,
                projected_at = $3
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(message_id.trim())
        .bind(delivery_state.trim())
        .bind(observed_at)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_telegram_message(row)?;
        capture_message_projection_observation_in_transaction(
            &mut transaction,
            "COMMUNICATION_MESSAGE",
            &updated,
            observed_at,
            "telegram_delivery_state_update",
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_message_id,
                "provider_chat_id": updated.provider_chat_id,
                "previous_delivery_state": current.delivery_state,
                "delivery_state": updated.delivery_state,
            }),
            "telegram.client.messages.provider_state.set_message_delivery_state",
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }

    pub(in crate::integrations::telegram) async fn apply_message_projection_update(
        &self,
        message_id: &str,
        body_text: &str,
        metadata: &Value,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        if !metadata.is_object() {
            return Err(TelegramError::InvalidRequest(
                "telegram message metadata must be a JSON object".to_owned(),
            ));
        }
        let Some(current) = self.message_by_id(message_id).await? else {
            return Ok(None);
        };

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET body_text = $2,
                message_metadata = $3,
                projected_at = $4
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(message_id.trim())
        .bind(body_text)
        .bind(metadata)
        .bind(observed_at)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_telegram_message(row)?;
        capture_message_projection_observation_in_transaction(
            &mut transaction,
            "COMMUNICATION_MESSAGE",
            &updated,
            observed_at,
            "telegram_content_projection_update",
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_message_id,
                "provider_chat_id": updated.provider_chat_id,
                "previous_body_text": current.text,
                "body_text": updated.text,
                "previous_metadata": current.metadata,
                "message_metadata": updated.metadata,
            }),
            "telegram.client.messages.provider_state.apply_message_projection_update",
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }

    pub(in crate::integrations::telegram) async fn apply_message_pinned_state(
        &self,
        message_id: &str,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        let Some(current) = self.message_by_id(message_id).await? else {
            return Ok(None);
        };

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = jsonb_set(
                    jsonb_set(
                        COALESCE(message_metadata, '{}'::jsonb),
                        '{pinned}',
                        to_jsonb($2::boolean),
                        true
                    ),
                    '{is_pinned}',
                    to_jsonb($2::boolean),
                    true
                ),
                projected_at = $3
            WHERE message_id = $1
            RETURNING
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            "#,
        )
        .bind(message_id.trim())
        .bind(is_pinned)
        .bind(observed_at)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };
        let updated = row_to_telegram_message(row)?;
        capture_message_projection_observation_in_transaction(
            &mut transaction,
            "COMMUNICATION_MESSAGE",
            &updated,
            observed_at,
            "telegram_pinned_state_update",
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_message_id,
                "provider_chat_id": updated.provider_chat_id,
                "previous_is_pinned": current.metadata.get("is_pinned").cloned().unwrap_or(Value::Bool(false)),
                "is_pinned": is_pinned,
                "message_metadata": updated.metadata,
            }),
            "telegram.client.messages.provider_state.apply_message_pinned_state",
        )
        .await?;
        transaction.commit().await?;
        Ok(Some(updated))
    }
}
