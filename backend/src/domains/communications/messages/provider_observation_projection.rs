use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::ProviderChannelMessageStore;
use crate::platform::communications::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage,
    ProviderCommunicationMessagePortError, ProviderMessageProjectionObservationContext,
};
use crate::platform::events::{EventStore, EventStoreError, NewEventEnvelope, StoredEventEnvelope};

pub const COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER: &str =
    "communication_provider_observation_projection";

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

pub async fn project_provider_observation_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    if !is_supported_provider_observation_event(&event.event.event_type) {
        return Ok(());
    }

    let updated = project_telegram_observation(pool.clone(), &event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
    if let Some(message) = updated {
        append_communication_message_updated_event(pool, &event, &message).await?;
    }

    Ok(())
}

fn is_supported_provider_observation_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "integration.telegram.message.content_observed"
            | "integration.telegram.message.metadata_observed"
            | "integration.telegram.message.delivery_state_observed"
            | "integration.telegram.message.pinned_state_observed"
            | "integration.telegram.attachment.download_state_observed"
    )
}

async fn project_telegram_observation(
    pool: PgPool,
    event: &StoredEventEnvelope,
) -> Result<Option<ProviderChannelMessage>, ProviderCommunicationMessagePortError> {
    let payload = &event.event.payload;
    let event_kind = required_str(payload, "event_kind")?;
    let message_id = required_str(payload, "message_id").or_else(|_| {
        event
            .event
            .subject
            .get("message_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ProviderCommunicationMessagePortError::InvalidRequest(
                    "provider observation is missing message_id".to_owned(),
                )
            })
    })?;
    let observed_at = parse_observed_at(payload)?;
    let fact_payload = payload.get("payload").ok_or_else(|| {
        ProviderCommunicationMessagePortError::InvalidRequest(
            "provider observation is missing payload".to_owned(),
        )
    })?;
    let store = ProviderChannelMessageStore::new(pool);
    let context = telegram_projection_context(event_kind);

    match event_kind {
        "metadata_observed" => {
            let metadata = fact_payload.get("message_metadata").ok_or_else(|| {
                ProviderCommunicationMessagePortError::InvalidRequest(
                    "metadata observation is missing message_metadata".to_owned(),
                )
            })?;
            store.apply_metadata(message_id, metadata, context).await
        }
        "delivery_state_observed" => {
            let delivery_state = required_str(fact_payload, "delivery_state")?;
            store
                .set_delivery_state(message_id, delivery_state, observed_at, context)
                .await
        }
        "content_observed" => {
            let body_text = required_str(fact_payload, "body_text")?;
            let metadata = fact_payload.get("message_metadata").ok_or_else(|| {
                ProviderCommunicationMessagePortError::InvalidRequest(
                    "content observation is missing message_metadata".to_owned(),
                )
            })?;
            store
                .apply_content_update(message_id, body_text, metadata, observed_at, context)
                .await
        }
        "pinned_state_observed" => {
            let is_pinned = fact_payload
                .get("is_pinned")
                .and_then(Value::as_bool)
                .ok_or_else(|| {
                    ProviderCommunicationMessagePortError::InvalidRequest(
                        "pin observation is missing is_pinned".to_owned(),
                    )
                })?;
            store
                .apply_pinned_state(message_id, is_pinned, observed_at, context)
                .await
        }
        "attachment_download_state_observed" => {
            let update = ProviderAttachmentDownloadStateUpdate {
                message_id,
                provider_attachment_id: required_str(fact_payload, "provider_attachment_id")?,
                provider_file_id: required_i64(fact_payload, "provider_file_id")?,
                download_state: required_str(fact_payload, "download_state")?,
                local_path: optional_str(fact_payload, "local_path"),
                size_bytes: optional_i64(fact_payload, "size_bytes"),
                content_type: required_str(fact_payload, "content_type")?,
                filename: optional_str(fact_payload, "filename"),
                observed_at,
                context,
            };
            store.update_attachment_download_state(update).await
        }
        other => Err(ProviderCommunicationMessagePortError::InvalidRequest(
            format!("unsupported provider observation event kind `{other}`"),
        )),
    }
}

fn telegram_projection_context(
    event_kind: &str,
) -> ProviderMessageProjectionObservationContext<'static> {
    ProviderMessageProjectionObservationContext {
        channel_kinds: TELEGRAM_CHANNEL_KINDS,
        relationship_kind: match event_kind {
            "metadata_observed" => "telegram_metadata_observed",
            "delivery_state_observed" => "telegram_delivery_state_observed",
            "content_observed" => "telegram_content_observed",
            "pinned_state_observed" => "telegram_pinned_state_observed",
            "attachment_download_state_observed" => "telegram_attachment_download_state_observed",
            _ => "telegram_provider_observed",
        },
        actor: "domains.communications.messages.communication_provider_observation_projection",
    }
}

async fn append_communication_message_updated_event(
    pool: PgPool,
    provider_event: &StoredEventEnvelope,
    message: &ProviderChannelMessage,
) -> Result<(), EventStoreError> {
    let event_id = format!(
        "evt_communication_message_updated_{}",
        provider_event.event.event_id
    );
    let event = NewEventEnvelope::builder(
        event_id,
        "communication.message.updated",
        Utc::now(),
        json!({
            "kind": "communication_projection",
            "consumer": COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        }),
        json!({
            "kind": "communication_message",
            "id": message.message_id,
        }),
    )
    .payload(json!({
        "message_id": message.message_id,
        "raw_record_id": message.raw_record_id,
        "account_id": message.account_id,
        "provider_record_id": message.provider_record_id,
        "channel_kind": message.channel_kind,
        "conversation_id": message.conversation_id,
        "delivery_state": message.delivery_state,
        "message_metadata": message.message_metadata,
        "provider_observation_event_id": provider_event.event.event_id,
        "provider_observation_event_type": provider_event.event.event_type,
    }))
    .provenance(json!({
        "ownership": "communications_projection",
        "source_event_id": provider_event.event.event_id,
    }))
    .causation_id(provider_event.event.event_id.clone())
    .correlation_id(
        provider_event
            .event
            .correlation_id
            .clone()
            .unwrap_or_else(|| provider_event.event.event_id.clone()),
    )
    .build()?;

    EventStore::new(pool).append_idempotent(&event).await?;
    Ok(())
}

fn parse_observed_at(
    payload: &Value,
) -> Result<DateTime<Utc>, ProviderCommunicationMessagePortError> {
    let Some(value) = payload.get("observed_at") else {
        return Ok(Utc::now());
    };
    let Some(value) = value.as_str() else {
        return Err(ProviderCommunicationMessagePortError::InvalidRequest(
            "observed_at must be an RFC3339 string".to_owned(),
        ));
    };
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "invalid observed_at: {error}"
            ))
        })
}

fn required_str<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a str, ProviderCommunicationMessagePortError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "{field} must be a non-empty string"
            ))
        })
}

fn optional_str<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn required_i64(value: &Value, field: &str) -> Result<i64, ProviderCommunicationMessagePortError> {
    value.get(field).and_then(Value::as_i64).ok_or_else(|| {
        ProviderCommunicationMessagePortError::InvalidRequest(format!("{field} must be an integer"))
    })
}

fn optional_i64(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}
