use chrono::{DateTime, Utc};
use serde_json::Value;

use super::super::errors::TelegramError;
use super::super::models::TelegramMessage;
use super::super::rows::provider_channel_message_to_telegram_message;
use super::super::store::TelegramStore;
use crate::platform::communications::{
    ProviderChannelMessageStore, ProviderMessageProjectionObservationContext,
};

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

impl TelegramStore {
    pub(in crate::integrations::telegram) async fn message_by_provider_message_id(
        &self,
        account_id: &str,
        provider_message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .message_by_provider_record_id(account_id, provider_message_id, TELEGRAM_CHANNEL_KINDS)
            .await?
            .map(provider_channel_message_to_telegram_message))
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
        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .apply_metadata(
                message_id,
                metadata,
                ProviderMessageProjectionObservationContext {
                    channel_kinds: TELEGRAM_CHANNEL_KINDS,
                    relationship_kind: "telegram_metadata_update",
                    actor: "telegram.client.messages.provider_state.apply_message_metadata",
                },
            )
            .await?
            .map(provider_channel_message_to_telegram_message))
    }

    pub(in crate::integrations::telegram) async fn set_message_delivery_state(
        &self,
        message_id: &str,
        delivery_state: &str,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .set_delivery_state(
                message_id,
                delivery_state,
                observed_at,
                ProviderMessageProjectionObservationContext {
                    channel_kinds: TELEGRAM_CHANNEL_KINDS,
                    relationship_kind: "telegram_delivery_state_update",
                    actor: "telegram.client.messages.provider_state.set_message_delivery_state",
                },
            )
            .await?
            .map(provider_channel_message_to_telegram_message))
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
        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .apply_content_update(
                message_id,
                body_text,
                metadata,
                observed_at,
                ProviderMessageProjectionObservationContext {
                    channel_kinds: TELEGRAM_CHANNEL_KINDS,
                    relationship_kind: "telegram_content_projection_update",
                    actor: "telegram.client.messages.provider_state.apply_message_projection_update",
                },
            )
            .await?
            .map(provider_channel_message_to_telegram_message))
    }

    pub(in crate::integrations::telegram) async fn apply_message_pinned_state(
        &self,
        message_id: &str,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .apply_pinned_state(
                message_id,
                is_pinned,
                observed_at,
                ProviderMessageProjectionObservationContext {
                    channel_kinds: TELEGRAM_CHANNEL_KINDS,
                    relationship_kind: "telegram_pinned_state_update",
                    actor: "telegram.client.messages.provider_state.apply_message_pinned_state",
                },
            )
            .await?
            .map(provider_channel_message_to_telegram_message))
    }
}
