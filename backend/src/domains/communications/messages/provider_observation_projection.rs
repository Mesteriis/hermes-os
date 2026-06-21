use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;

use super::ProviderChannelMessageStore;
use crate::platform::communications::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage,
    ProviderCommunicationMessagePortError, ProviderMessageObservationProjectionPort,
    ProviderMessageProjectionObservationContext,
};
use crate::platform::events::{EventStore, NewEventEnvelope};

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

pub async fn record_telegram_message_metadata_observation(
    pool: PgPool,
    message_id: &str,
    metadata: &Value,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    let payload = json!({ "message_metadata": metadata });
    let Some((store, should_project)) =
        prepare_telegram_observation(pool, message_id, "metadata_observed", &payload, Utc::now())
            .await?
    else {
        return Ok(None);
    };
    if !should_project {
        return store
            .message_by_id(message_id, TELEGRAM_CHANNEL_KINDS)
            .await;
    }
    store
        .apply_metadata(
            message_id,
            metadata,
            telegram_projection_context(
                "telegram_metadata_observed",
                "domains.communications.messages.record_telegram_message_metadata_observation",
            ),
        )
        .await
}

pub async fn record_telegram_message_delivery_observation(
    pool: PgPool,
    message_id: &str,
    delivery_state: &str,
    observed_at: DateTime<Utc>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    let payload = json!({
        "delivery_state": delivery_state,
        "observed_at": observed_at,
    });
    let Some((store, should_project)) = prepare_telegram_observation(
        pool,
        message_id,
        "delivery_state_observed",
        &payload,
        observed_at,
    )
    .await?
    else {
        return Ok(None);
    };
    if !should_project {
        return store
            .message_by_id(message_id, TELEGRAM_CHANNEL_KINDS)
            .await;
    }
    store
        .set_delivery_state(
            message_id,
            delivery_state,
            observed_at,
            telegram_projection_context(
                "telegram_delivery_state_observed",
                "domains.communications.messages.record_telegram_message_delivery_observation",
            ),
        )
        .await
}

pub async fn record_telegram_message_content_observation(
    pool: PgPool,
    message_id: &str,
    body_text: &str,
    metadata: &Value,
    observed_at: DateTime<Utc>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    let payload = json!({
        "body_text": body_text,
        "message_metadata": metadata,
        "observed_at": observed_at,
    });
    let Some((store, should_project)) =
        prepare_telegram_observation(pool, message_id, "content_observed", &payload, observed_at)
            .await?
    else {
        return Ok(None);
    };
    if !should_project {
        return store
            .message_by_id(message_id, TELEGRAM_CHANNEL_KINDS)
            .await;
    }
    store
        .apply_content_update(
            message_id,
            body_text,
            metadata,
            observed_at,
            telegram_projection_context(
                "telegram_content_observed",
                "domains.communications.messages.record_telegram_message_content_observation",
            ),
        )
        .await
}

pub async fn record_telegram_message_pin_observation(
    pool: PgPool,
    message_id: &str,
    is_pinned: bool,
    observed_at: DateTime<Utc>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    let payload = json!({
        "is_pinned": is_pinned,
        "observed_at": observed_at,
    });
    let Some((store, should_project)) = prepare_telegram_observation(
        pool,
        message_id,
        "pinned_state_observed",
        &payload,
        observed_at,
    )
    .await?
    else {
        return Ok(None);
    };
    if !should_project {
        return store
            .message_by_id(message_id, TELEGRAM_CHANNEL_KINDS)
            .await;
    }
    store
        .apply_pinned_state(
            message_id,
            is_pinned,
            observed_at,
            telegram_projection_context(
                "telegram_pinned_state_observed",
                "domains.communications.messages.record_telegram_message_pin_observation",
            ),
        )
        .await
}

pub async fn record_telegram_attachment_download_observation(
    pool: PgPool,
    update: ProviderAttachmentDownloadStateUpdate<'_>,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    let payload = json!({
        "provider_attachment_id": update.provider_attachment_id,
        "provider_file_id": update.provider_file_id,
        "download_state": update.download_state,
        "local_path": update.local_path,
        "size_bytes": update.size_bytes,
        "content_type": update.content_type,
        "filename": update.filename,
        "observed_at": update.observed_at,
    });
    let Some((store, should_project)) = prepare_telegram_observation(
        pool,
        update.message_id,
        "attachment_download_state_observed",
        &payload,
        update.observed_at,
    )
    .await?
    else {
        return Ok(None);
    };
    if !should_project {
        return store
            .message_by_id(update.message_id, TELEGRAM_CHANNEL_KINDS)
            .await;
    }
    store.update_attachment_download_state(update).await
}

async fn prepare_telegram_observation(
    pool: PgPool,
    message_id: &str,
    event_kind: &str,
    payload: &Value,
    observed_at: DateTime<Utc>,
) -> Result<Option<(ProviderChannelMessageStore, bool)>, ProviderCommunicationMessagePortError> {
    let store = ProviderChannelMessageStore::new(pool.clone());
    let Some(current) = store
        .message_by_id(message_id, TELEGRAM_CHANNEL_KINDS)
        .await?
    else {
        return Ok(None);
    };
    let event = telegram_provider_observation_event(&current, event_kind, payload, observed_at)?;
    let appended = EventStore::new(pool).append_idempotent(&event).await?;
    Ok(Some((store, appended.is_some())))
}

fn telegram_projection_context(
    relationship_kind: &'static str,
    actor: &'static str,
) -> ProviderMessageProjectionObservationContext<'static> {
    ProviderMessageProjectionObservationContext {
        channel_kinds: TELEGRAM_CHANNEL_KINDS,
        relationship_kind,
        actor,
    }
}

fn telegram_provider_observation_event(
    message: &ProviderChannelMessage,
    event_kind: &str,
    payload: &Value,
    observed_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, ProviderCommunicationMessagePortError> {
    let payload_hash = sha256_json(payload)?;
    let source_id = format!(
        "telegram:{}:{}:{}:{}",
        message.account_id, event_kind, message.provider_record_id, payload_hash
    );

    Ok(NewEventEnvelope::builder(
        format!("evt_provider_observation_{}", source_id.replace(':', "_")),
        format!("integration.telegram.message.{event_kind}"),
        observed_at,
        json!({
            "kind": "provider_observation",
            "provider": "telegram",
            "account_id": message.account_id,
            "source_id": source_id,
        }),
        json!({
            "kind": "provider_message",
            "provider": "telegram",
            "id": message.provider_record_id,
            "message_id": message.message_id,
        }),
    )
    .payload(json!({
        "provider_kind": message.channel_kind,
        "account_id": message.account_id,
        "external_event_id": Value::Null,
        "external_message_id": message.provider_record_id,
        "event_kind": event_kind,
        "observed_at": observed_at,
        "payload_hash": payload_hash,
        "payload": payload,
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "ownership": "provider_observation_fact",
    }))
    .correlation_id(source_id)
    .build()?)
}

fn sha256_json(value: &Value) -> Result<String, ProviderCommunicationMessagePortError> {
    let encoded = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

impl ProviderMessageObservationProjectionPort for ProviderChannelMessageStore {
    fn record_telegram_message_metadata_observation<'a>(
        &'a self,
        message_id: &'a str,
        metadata: &'a Value,
    ) -> crate::platform::communications::ProviderChannelMessagePortFuture<
        'a,
        Option<ProviderChannelMessage>,
    > {
        Box::pin(async move {
            record_telegram_message_metadata_observation(self.clone_pool(), message_id, metadata)
                .await
        })
    }

    fn record_telegram_message_delivery_observation<'a>(
        &'a self,
        message_id: &'a str,
        delivery_state: &'a str,
        observed_at: DateTime<Utc>,
    ) -> crate::platform::communications::ProviderChannelMessagePortFuture<
        'a,
        Option<ProviderChannelMessage>,
    > {
        Box::pin(async move {
            record_telegram_message_delivery_observation(
                self.clone_pool(),
                message_id,
                delivery_state,
                observed_at,
            )
            .await
        })
    }

    fn record_telegram_message_content_observation<'a>(
        &'a self,
        message_id: &'a str,
        body_text: &'a str,
        metadata: &'a Value,
        observed_at: DateTime<Utc>,
    ) -> crate::platform::communications::ProviderChannelMessagePortFuture<
        'a,
        Option<ProviderChannelMessage>,
    > {
        Box::pin(async move {
            record_telegram_message_content_observation(
                self.clone_pool(),
                message_id,
                body_text,
                metadata,
                observed_at,
            )
            .await
        })
    }

    fn record_telegram_message_pin_observation<'a>(
        &'a self,
        message_id: &'a str,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
    ) -> crate::platform::communications::ProviderChannelMessagePortFuture<
        'a,
        Option<ProviderChannelMessage>,
    > {
        Box::pin(async move {
            record_telegram_message_pin_observation(
                self.clone_pool(),
                message_id,
                is_pinned,
                observed_at,
            )
            .await
        })
    }

    fn record_telegram_attachment_download_observation<'a>(
        &'a self,
        update: ProviderAttachmentDownloadStateUpdate<'a>,
    ) -> crate::platform::communications::ProviderChannelMessagePortFuture<
        'a,
        Option<ProviderChannelMessage>,
    > {
        Box::pin(async move {
            record_telegram_attachment_download_observation(self.clone_pool(), update).await
        })
    }
}
