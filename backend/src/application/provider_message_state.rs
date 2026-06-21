use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::domains::communications::messages::ProviderChannelMessageStore;
use crate::platform::communications::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage,
    ProviderCommunicationMessagePortError, ProviderMessageProjectionObservationContext,
};

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

pub(crate) async fn observe_telegram_message_metadata(
    pool: PgPool,
    message_id: &str,
    metadata: &Value,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    ProviderChannelMessageStore::new(pool)
        .apply_metadata(
            message_id,
            metadata,
            ProviderMessageProjectionObservationContext {
                channel_kinds: TELEGRAM_CHANNEL_KINDS,
                relationship_kind: "telegram_metadata_observed",
                actor: "application.provider_message_state.observe_telegram_message_metadata",
            },
        )
        .await
}

pub(crate) async fn observe_telegram_message_delivery(
    pool: PgPool,
    message_id: &str,
    delivery_state: &str,
    observed_at: DateTime<Utc>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    ProviderChannelMessageStore::new(pool)
        .set_delivery_state(
            message_id,
            delivery_state,
            observed_at,
            ProviderMessageProjectionObservationContext {
                channel_kinds: TELEGRAM_CHANNEL_KINDS,
                relationship_kind: "telegram_delivery_state_observed",
                actor: "application.provider_message_state.observe_telegram_message_delivery",
            },
        )
        .await
}

pub(crate) async fn observe_telegram_message_content(
    pool: PgPool,
    message_id: &str,
    body_text: &str,
    metadata: &Value,
    observed_at: DateTime<Utc>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    ProviderChannelMessageStore::new(pool)
        .apply_content_update(
            message_id,
            body_text,
            metadata,
            observed_at,
            ProviderMessageProjectionObservationContext {
                channel_kinds: TELEGRAM_CHANNEL_KINDS,
                relationship_kind: "telegram_content_observed",
                actor: "application.provider_message_state.observe_telegram_message_content",
            },
        )
        .await
}

pub(crate) async fn observe_telegram_message_pin_state(
    pool: PgPool,
    message_id: &str,
    is_pinned: bool,
    observed_at: DateTime<Utc>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    ProviderChannelMessageStore::new(pool)
        .apply_pinned_state(
            message_id,
            is_pinned,
            observed_at,
            ProviderMessageProjectionObservationContext {
                channel_kinds: TELEGRAM_CHANNEL_KINDS,
                relationship_kind: "telegram_pinned_state_observed",
                actor: "application.provider_message_state.observe_telegram_message_pin_state",
            },
        )
        .await
}

pub(crate) async fn observe_telegram_attachment_download(
    pool: PgPool,
    update: ProviderAttachmentDownloadStateUpdate<'_>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    ProviderChannelMessageStore::new(pool)
        .update_attachment_download_state(update)
        .await
}
