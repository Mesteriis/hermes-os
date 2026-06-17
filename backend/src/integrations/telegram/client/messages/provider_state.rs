use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;

use crate::domains::mail::messages::MessageProjectionStore;

use super::super::errors::TelegramError;
use super::super::models::TelegramMessage;
use super::super::rows::row_to_telegram_message;
use super::super::store::TelegramStore;

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
        MessageProjectionStore::new(self.pool.clone())
            .set_message_metadata(message_id, metadata)
            .await?;
        self.message_by_id(message_id).await
    }

    pub(in crate::integrations::telegram) async fn set_message_delivery_state(
        &self,
        message_id: &str,
        delivery_state: &str,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
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
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_message).transpose()
    }

    pub(in crate::integrations::telegram) async fn apply_message_projection_update(
        &self,
        message_id: &str,
        body_text: &str,
        metadata: &Value,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
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
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_message).transpose()
    }

    pub(in crate::integrations::telegram) async fn apply_message_pinned_state(
        &self,
        message_id: &str,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
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
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_message).transpose()
    }
}
