use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::{
    CommunicationMessageProjectionPort, MessageProjectionError, NewProjectedMessage,
    ProjectedMessage, ProviderChannelMessageStore, project_raw_email_message,
    project_raw_email_message_from_blob,
};
use crate::domains::communications::core::CommunicationIngestionPort;
use crate::domains::communications::delivery_notifications::consume_accepted_mail_delivery_signal;
use crate::domains::communications::storage::LocalCommunicationBlobStore;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::communications::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage,
    ProviderCommunicationMessagePortError, ProviderMessageProjectionObservationContext,
};
use crate::platform::events::{
    EventEnvelope, EventStore, EventStoreError, NewEventEnvelope, StoredEventEnvelope,
};

pub const COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER: &str =
    "communication_provider_observation_projection";

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

pub async fn project_provider_observation_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    if is_supported_mail_delivery_signal_event(&event.event.event_type) {
        consume_accepted_mail_delivery_signal(pool.clone(), &event.event)
            .await
            .map(|_| ())
            .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
        return Ok(());
    }

    if is_base_accepted_signal_event(&event.event.event_type) {
        consume_accepted_signal_event(pool.clone(), &event.event)
            .await
            .map(|_| ())
            .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))?;
        return Ok(());
    }

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

pub async fn replay_accepted_signal_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_provider_observation_event(pool, event).await
}

fn is_base_accepted_signal_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.mail.message"
            | "signal.accepted.telegram.message"
            | "signal.accepted.whatsapp.message"
    )
}

fn is_supported_mail_delivery_signal_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.mail.delivery_status" | "signal.accepted.mail.read_receipt"
    )
}

fn is_supported_provider_observation_event(event_type: &str) -> bool {
    matches!(
        event_type,
        "signal.accepted.telegram.message.content"
            | "signal.accepted.telegram.message.metadata"
            | "signal.accepted.telegram.message.delivery_state"
            | "signal.accepted.telegram.message.pinned_state"
            | "signal.accepted.telegram.attachment.download_state"
    )
}

pub fn supports_communication_projection_signal_event(event_type: &str) -> bool {
    is_base_accepted_signal_event(event_type) || is_supported_provider_observation_event(event_type)
}

pub async fn project_accepted_signal_if_runtime_allows(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    if !accepted_signal_projection_runtime_allows(&pool).await? {
        return Ok(None);
    }

    consume_accepted_signal_event(pool, event).await
}

pub async fn consume_accepted_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    if event.event_type == "signal.accepted.mail.message" {
        return project_mail_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.telegram.message" {
        return project_telegram_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.whatsapp.message" {
        return project_whatsapp_signal_event(pool, event).await;
    }

    Ok(None)
}

async fn project_accepted_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    consume_accepted_signal_event(pool, event).await
}

async fn accepted_signal_projection_runtime_allows(
    pool: &PgPool,
) -> Result<bool, CommunicationSignalProjectionError> {
    Ok(crate::platform::events::runtime_allows_processing(
        pool,
        "system",
        COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        &json!({
            "label": "Communications accepted-signal consumer",
            "scope": "consumer",
        }),
    )
    .await?)
}

async fn project_mail_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.mail.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionPort::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let message_store = CommunicationMessageProjectionPort::new(pool);

    if raw_record.payload.get("raw_blob_storage_path").is_some() {
        let blob_store = LocalCommunicationBlobStore::new(mail_blob_root_from_event(event));
        Ok(Some(
            project_raw_email_message_from_blob(&message_store, &blob_store, &raw_record).await?,
        ))
    } else {
        Ok(Some(
            project_raw_email_message(&message_store, &raw_record).await?,
        ))
    }
}

async fn project_whatsapp_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.whatsapp.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionPort::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let provider_chat_id = required_payload_str(&raw_record.payload, "provider_chat_id")?;
    let chat_title = required_payload_str(&raw_record.payload, "chat_title")?;
    let sender_display_name = required_payload_str(&raw_record.payload, "sender_display_name")?;
    let body_text = required_payload_str(&raw_record.payload, "text")?;
    let delivery_state = required_payload_str(&raw_record.payload, "delivery_state")?;

    Ok(Some(
        CommunicationMessageProjectionPort::new(pool)
            .upsert_channel_message(&NewProjectedMessage {
                message_id: whatsapp_web_message_id(
                    &raw_record.account_id,
                    &raw_record.provider_record_id,
                ),
                raw_record_id: raw_record.raw_record_id.clone(),
                account_id: raw_record.account_id.clone(),
                provider_record_id: raw_record.provider_record_id.clone(),
                subject: chat_title,
                sender: sender_display_name.clone(),
                recipients: vec![provider_chat_id.clone()],
                body_text,
                occurred_at: raw_record.occurred_at,
                channel_kind: "whatsapp_web".to_owned(),
                conversation_id: Some(provider_chat_id),
                sender_display_name: Some(sender_display_name),
                delivery_state,
                message_metadata: raw_record.payload,
            })
            .await?,
    ))
}

async fn project_telegram_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.telegram.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionPort::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let provider_chat_id = required_payload_str(&raw_record.payload, "provider_chat_id")?;
    let chat_title = required_payload_str(&raw_record.payload, "chat_title")?;
    let sender_display_name = required_payload_str(&raw_record.payload, "sender_display_name")?;
    let body_text = optional_payload_str(&raw_record.payload, "text").unwrap_or_default();
    let delivery_state = required_payload_str(&raw_record.payload, "delivery_state")?;
    let channel_kind = raw_record
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .unwrap_or("telegram_user")
        .trim()
        .to_owned();
    let allow_empty_body_text = body_text.is_empty()
        && raw_record
            .provenance
            .get("runtime")
            .and_then(Value::as_str)
            .map(str::trim)
            == Some("tdlib")
        && raw_record.payload.get("tdlib_raw").is_some();

    let message = NewProjectedMessage {
        message_id: telegram_message_id(&raw_record.account_id, &raw_record.provider_record_id),
        raw_record_id: raw_record.raw_record_id.clone(),
        account_id: raw_record.account_id.clone(),
        provider_record_id: raw_record.provider_record_id.clone(),
        subject: chat_title,
        sender: sender_display_name.clone(),
        recipients: vec![provider_chat_id.clone()],
        body_text,
        occurred_at: raw_record.occurred_at,
        channel_kind,
        conversation_id: Some(provider_chat_id),
        sender_display_name: Some(sender_display_name),
        delivery_state,
        message_metadata: raw_record.payload,
    };

    let projected = if allow_empty_body_text {
        CommunicationMessageProjectionPort::new(pool)
            .upsert_channel_message_allowing_empty_body_text(&message)
            .await?
    } else {
        CommunicationMessageProjectionPort::new(pool)
            .upsert_channel_message(&message)
            .await?
    };

    Ok(Some(projected))
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

fn required_payload_str(
    value: &Value,
    field: &'static str,
) -> Result<String, CommunicationSignalProjectionError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or(CommunicationSignalProjectionError::Message(
            MessageProjectionError::MissingPayloadField(field),
        ))
}

fn optional_payload_str(value: &Value, field: &'static str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn required_subject_str<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, CommunicationSignalProjectionError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(CommunicationSignalProjectionError::MissingSubjectField(
            field,
        ))
}

fn mail_blob_root_from_event(event: &EventEnvelope) -> &Path {
    event
        .provenance
        .get("raw_event_provenance")
        .and_then(|value| value.get("blob_root"))
        .and_then(Value::as_str)
        .map(Path::new)
        .unwrap_or_else(|| Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT))
}

fn whatsapp_web_message_id(account_id: &str, provider_message_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_message_id.as_bytes());
    format!("message:v5:whatsapp_web:{:x}", hasher.finalize())
}

fn telegram_message_id(account_id: &str, provider_message_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_message_id.as_bytes());
    format!("message:v4:telegram:{:x}", hasher.finalize())
}

#[derive(Debug, Error)]
pub enum CommunicationSignalProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Communication(#[from] crate::domains::communications::core::CommunicationIngestionError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error("accepted signal subject is missing `{0}`")]
    MissingSubjectField(&'static str),
}
