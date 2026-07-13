use hermes_events_api::{EventEnvelope, NewEventEnvelope, StoredEventEnvelope};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use super::rows::row_to_projected_message;
use super::{
    MessageProjectionError, MessageProjectionStore, NewProjectedMessage, ProjectedMessage,
    ProviderChannelMessageStore, project_raw_email_message, project_raw_email_message_from_blob,
};
use crate::domains::communications::delivery_notifications::consume_accepted_mail_delivery_signal;
use crate::domains::communications::storage::LocalCommunicationBlobStore;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::platform::communications::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage,
    ProviderCommunicationMessagePortError, ProviderMessageProjectionObservationContext,
};
use hermes_communications_api::evidence::StoredRawCommunicationRecord;
use hermes_communications_postgres::store::CommunicationIngestionStore;
use hermes_events_postgres::errors::EventStoreError;
use hermes_events_postgres::store::EventStore;

pub const COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER: &str =
    "communication_provider_observation_projection";

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];
const WHATSAPP_CHANNEL_KINDS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];
const ZULIP_CHANNEL_KIND: &str = "zulip";

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
            | "signal.accepted.zulip.message"
            | "signal.accepted.zulip.reaction"
            | "signal.accepted.zulip.message_update"
            | "signal.accepted.zulip.message_delete"
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
            | "signal.accepted.telegram.message.provider_identity"
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
    let Some(projection) = project_accepted_signal_event(pool.clone(), event).await? else {
        return Ok(None);
    };

    append_communication_message_projected_event(
        pool,
        event,
        &projection.message,
        projection.message_existed,
    )
    .await?;

    Ok(Some(projection.message))
}

struct AcceptedSignalProjection {
    message: ProjectedMessage,
    message_existed: bool,
}

async fn project_accepted_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type == "signal.accepted.mail.message" {
        return project_mail_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.telegram.message" {
        return project_telegram_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.whatsapp.message" {
        return project_whatsapp_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.zulip.message" {
        return project_zulip_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.zulip.reaction" {
        return project_zulip_reaction_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.zulip.message_update" {
        return project_zulip_message_update_signal_event(pool, event).await;
    }
    if event.event_type == "signal.accepted.zulip.message_delete" {
        return project_zulip_message_delete_signal_event(pool, event).await;
    }

    Ok(None)
}

async fn accepted_signal_projection_runtime_allows(
    pool: &PgPool,
) -> Result<bool, CommunicationSignalProjectionError> {
    Ok(crate::platform::events::runtime::runtime_allows_processing(
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
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.mail.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;
    let message_store = MessageProjectionStore::new(pool);

    let message = if raw_record.payload.get("raw_blob_storage_path").is_some() {
        let blob_store = LocalCommunicationBlobStore::new(mail_blob_root_from_event(event));
        project_raw_email_message_from_blob(&message_store, &blob_store, &raw_record).await?
    } else {
        project_raw_email_message(&message_store, &raw_record).await?
    };

    Ok(Some(AcceptedSignalProjection {
        message,
        message_existed,
    }))
}

async fn project_whatsapp_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.whatsapp.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    let provider_chat_id = required_payload_str(&raw_record.payload, "provider_chat_id")?;
    let chat_title = required_payload_str(&raw_record.payload, "chat_title")?;
    let sender_display_name = required_payload_str(&raw_record.payload, "sender_display_name")?;
    let body_text = required_payload_str(&raw_record.payload, "text")?;
    let delivery_state = required_payload_str(&raw_record.payload, "delivery_state")?;
    let channel_kind = raw_record
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| matches!(*value, "whatsapp_web" | "whatsapp_business_cloud"))
        .unwrap_or("whatsapp_web")
        .to_owned();
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;

    let message = MessageProjectionStore::new(pool)
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
            channel_kind,
            conversation_id: Some(provider_chat_id),
            sender_display_name: Some(sender_display_name),
            delivery_state,
            message_metadata: raw_record.payload,
        })
        .await?;

    Ok(Some(AcceptedSignalProjection {
        message,
        message_existed,
    }))
}

async fn project_telegram_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.telegram.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
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
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;

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
        MessageProjectionStore::new(pool)
            .upsert_channel_message_allowing_empty_body_text(&message)
            .await?
    } else {
        MessageProjectionStore::new(pool)
            .upsert_channel_message(&message)
            .await?
    };

    Ok(Some(AcceptedSignalProjection {
        message: projected,
        message_existed,
    }))
}

async fn project_zulip_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.zulip.message" {
        return Ok(None);
    }

    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()))?;
    ensure_canonical_communication_account(&pool, &raw_record.account_id).await?;
    let provider_message_id = required_payload_str(&raw_record.payload, "provider_message_id")?;
    let body_text = required_payload_str(&raw_record.payload, "content")?;
    let sender_display_name = optional_payload_str(&raw_record.payload, "sender_full_name")
        .or_else(|| optional_payload_str(&raw_record.payload, "sender_email"))
        .unwrap_or_else(|| "Zulip sender".to_owned());
    let delivery_state = optional_payload_str(&raw_record.payload, "delivery_state")
        .unwrap_or_else(|| "received".to_owned());
    let target = zulip_message_target(&raw_record.account_id, &raw_record.payload);
    let message_existed = communication_message_exists(
        &pool,
        &raw_record.account_id,
        &raw_record.provider_record_id,
    )
    .await?;

    let message = MessageProjectionStore::new(pool)
        .upsert_channel_message(&NewProjectedMessage {
            message_id: zulip_message_id(&raw_record.account_id, &provider_message_id),
            raw_record_id: raw_record.raw_record_id.clone(),
            account_id: raw_record.account_id.clone(),
            provider_record_id: raw_record.provider_record_id.clone(),
            subject: target.subject,
            sender: sender_display_name.clone(),
            recipients: target.recipients,
            body_text,
            occurred_at: raw_record.occurred_at,
            channel_kind: ZULIP_CHANNEL_KIND.to_owned(),
            conversation_id: Some(target.conversation_id),
            sender_display_name: Some(sender_display_name),
            delivery_state,
            message_metadata: raw_record.payload,
        })
        .await?;

    Ok(Some(AcceptedSignalProjection {
        message,
        message_existed,
    }))
}

async fn project_zulip_reaction_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.zulip.reaction" {
        return Ok(None);
    }

    let raw_record = raw_record_for_accepted_signal(pool.clone(), event).await?;
    let message = zulip_target_message(pool.clone(), &raw_record).await?;
    let reaction = optional_payload_str(&raw_record.payload, "emoji_name")
        .or_else(|| optional_payload_str(&raw_record.payload, "emoji_code"))
        .ok_or(CommunicationSignalProjectionError::Message(
            MessageProjectionError::MissingPayloadField("emoji_name"),
        ))?;
    let provider_actor_id = optional_payload_str(&raw_record.payload, "provider_actor_id")
        .or_else(|| optional_payload_str(&raw_record.payload, "sender_email"))
        .unwrap_or_else(|| "unknown".to_owned());
    let sender_display_name = optional_payload_str(&raw_record.payload, "sender_display_name")
        .or_else(|| optional_payload_str(&raw_record.payload, "sender_email"))
        .unwrap_or_else(|| provider_actor_id.clone());
    let reaction_op = optional_payload_str(&raw_record.payload, "reaction_op")
        .unwrap_or_else(|| "add".to_owned());
    let is_active = !matches!(reaction_op.as_str(), "remove" | "delete");
    let observed_at = zulip_observed_at(&raw_record, event);
    let reaction_id = zulip_reaction_id(
        &message.account_id,
        &message.provider_record_id,
        &provider_actor_id,
        &reaction,
    );

    sqlx::query(
        r#"
        INSERT INTO communication_message_reactions (
            reaction_id, message_id, account_id, provider_message_id,
            provider_conversation_id, sender_display_name, reaction, is_active,
            observed_at, source_event, provider_actor_id, metadata, provenance, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now())
        ON CONFLICT (reaction_id)
        DO UPDATE SET
            sender_display_name = EXCLUDED.sender_display_name,
            reaction = EXCLUDED.reaction,
            is_active = EXCLUDED.is_active,
            observed_at = EXCLUDED.observed_at,
            source_event = EXCLUDED.source_event,
            provider_actor_id = EXCLUDED.provider_actor_id,
            metadata = EXCLUDED.metadata,
            provenance = EXCLUDED.provenance,
            updated_at = now()
        "#,
    )
    .bind(&reaction_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_record_id)
    .bind(&message.conversation_id)
    .bind(&sender_display_name)
    .bind(&reaction)
    .bind(is_active)
    .bind(observed_at)
    .bind(&event.event_id)
    .bind(&provider_actor_id)
    .bind(json!({
        "provider": "zulip",
        "provider_event_id": raw_record.payload.get("provider_event_id"),
        "provider_event_type": raw_record.payload.get("provider_event_type"),
        "reaction_type": raw_record.payload.get("reaction_type"),
        "reaction_op": reaction_op,
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .bind(json!({
        "provider": "zulip",
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .execute(&pool)
    .await?;

    let projected = projected_message_by_id(&pool, &message.message_id)
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip reaction target `{}` disappeared during projection",
                message.provider_record_id
            ))
        })?;

    Ok(Some(AcceptedSignalProjection {
        message: projected,
        message_existed: true,
    }))
}

async fn project_zulip_message_update_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.zulip.message_update" {
        return Ok(None);
    }

    let raw_record = raw_record_for_accepted_signal(pool.clone(), event).await?;
    let message = zulip_target_message(pool.clone(), &raw_record).await?;
    let body_text = required_payload_str(&raw_record.payload, "content")?;
    let observed_at = zulip_observed_at(&raw_record, event);
    let updated_metadata = merged_zulip_message_metadata(
        &message.message_metadata,
        json!({
            "edited": true,
            "provider": "zulip",
            "provider_event_id": raw_record.payload.get("provider_event_id"),
            "provider_event_type": raw_record.payload.get("provider_event_type"),
            "raw_record_id": &raw_record.raw_record_id,
            "accepted_signal_event_id": &event.event_id,
            "prev_content": raw_record.payload.get("prev_content"),
            "topic": raw_record.payload.get("topic"),
            "prev_topic": raw_record.payload.get("prev_topic"),
            "edit_timestamp": raw_record.payload.get("edit_timestamp"),
        }),
    )?;
    let updated_message = ProviderChannelMessageStore::new(pool.clone())
        .apply_content_update(
            &message.message_id,
            &body_text,
            &updated_metadata,
            observed_at,
            zulip_projection_context("zulip_content_observed"),
        )
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip update target `{}` disappeared during projection",
                message.provider_record_id
            ))
        })?;

    let version_id = zulip_message_version_id(&event.event_id);
    let next_version_number: i32 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(MAX(version_number), 0) + 1
        FROM communication_message_versions
        WHERE message_id = $1
        "#,
    )
    .bind(&message.message_id)
    .fetch_one(&pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO communication_message_versions (
            version_id, message_id, account_id, provider_message_id,
            provider_conversation_id, version_number, body_text, edited_at,
            source_event, diff_payload, provenance
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (version_id) DO NOTHING
        "#,
    )
    .bind(&version_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_record_id)
    .bind(&message.conversation_id)
    .bind(next_version_number)
    .bind(&body_text)
    .bind(observed_at)
    .bind(&event.event_id)
    .bind(zulip_content_diff(
        Some(message.body_text.as_str()),
        body_text.as_str(),
    ))
    .bind(json!({
        "provider": "zulip",
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .execute(&pool)
    .await?;

    let projected = projected_message_by_id(&pool, &updated_message.message_id)
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip update target `{}` disappeared after projection",
                message.provider_record_id
            ))
        })?;

    Ok(Some(AcceptedSignalProjection {
        message: projected,
        message_existed: true,
    }))
}

async fn project_zulip_message_delete_signal_event(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<Option<AcceptedSignalProjection>, CommunicationSignalProjectionError> {
    if event.event_type != "signal.accepted.zulip.message_delete" {
        return Ok(None);
    }

    let raw_record = raw_record_for_accepted_signal(pool.clone(), event).await?;
    let message = zulip_target_message(pool.clone(), &raw_record).await?;
    let observed_at = zulip_observed_at(&raw_record, event);
    let tombstone_id = zulip_message_tombstone_id(&event.event_id);

    sqlx::query(
        r#"
        INSERT INTO communication_message_tombstones (
            tombstone_id, message_id, account_id, provider_message_id,
            provider_conversation_id, reason_class, actor_class, observed_at,
            source_event, is_provider_delete, is_local_visible, metadata, provenance
        )
        VALUES ($1, $2, $3, $4, $5, 'deleted_by_provider', 'provider', $6, $7, TRUE, FALSE, $8, $9)
        ON CONFLICT (tombstone_id) DO NOTHING
        "#,
    )
    .bind(&tombstone_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_record_id)
    .bind(&message.conversation_id)
    .bind(observed_at)
    .bind(&event.event_id)
    .bind(json!({
        "provider": "zulip",
        "provider_event_id": raw_record.payload.get("provider_event_id"),
        "provider_event_type": raw_record.payload.get("provider_event_type"),
        "message_type": raw_record.payload.get("message_type"),
        "stream_id": raw_record.payload.get("stream_id"),
        "topic": raw_record.payload.get("topic"),
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .bind(json!({
        "provider": "zulip",
        "raw_record_id": &raw_record.raw_record_id,
        "accepted_signal_event_id": &event.event_id,
    }))
    .execute(&pool)
    .await?;

    let projected = projected_message_by_id(&pool, &message.message_id)
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip delete target `{}` disappeared during projection",
                message.provider_record_id
            ))
        })?;

    Ok(Some(AcceptedSignalProjection {
        message: projected,
        message_existed: true,
    }))
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
        "provider_identity_observed" => {
            let provider_record_id = required_str(fact_payload, "provider_record_id")?;
            store
                .rebind_provider_record_id(message_id, provider_record_id, observed_at, context)
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
                communication_attachment_id: optional_str(
                    fact_payload,
                    "communication_attachment_id",
                ),
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

pub async fn project_whatsapp_content_observed(
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
            whatsapp_projection_context("whatsapp_content_observed"),
        )
        .await
}

pub async fn project_whatsapp_delivery_state_observed(
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
            whatsapp_projection_context("whatsapp_delivery_state_observed"),
        )
        .await
}

fn telegram_projection_context(
    event_kind: &str,
) -> ProviderMessageProjectionObservationContext<'static> {
    ProviderMessageProjectionObservationContext {
        channel_kinds: TELEGRAM_CHANNEL_KINDS,
        relationship_kind: match event_kind {
            "metadata_observed" => "telegram_metadata_observed",
            "delivery_state_observed" => "telegram_delivery_state_observed",
            "provider_identity_observed" => "telegram_provider_identity_observed",
            "content_observed" => "telegram_content_observed",
            "pinned_state_observed" => "telegram_pinned_state_observed",
            "attachment_download_state_observed" => "telegram_attachment_download_state_observed",
            _ => "telegram_provider_observed",
        },
        actor: "domains.communications.messages.communication_provider_observation_projection",
    }
}

fn whatsapp_projection_context(
    relationship_kind: &'static str,
) -> ProviderMessageProjectionObservationContext<'static> {
    ProviderMessageProjectionObservationContext {
        channel_kinds: WHATSAPP_CHANNEL_KINDS,
        relationship_kind,
        actor: "domains.communications.messages.communication_provider_observation_projection",
    }
}

fn zulip_projection_context(
    relationship_kind: &'static str,
) -> ProviderMessageProjectionObservationContext<'static> {
    ProviderMessageProjectionObservationContext {
        channel_kinds: &[ZULIP_CHANNEL_KIND],
        relationship_kind,
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
            "entity_id": message.message_id,
            "message_id": message.message_id,
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

async fn append_communication_message_projected_event(
    pool: PgPool,
    accepted_event: &EventEnvelope,
    message: &ProjectedMessage,
    message_existed: bool,
) -> Result<(), EventStoreError> {
    let event_name = if message_existed {
        "communication.message.updated"
    } else {
        "communication.message.recorded"
    };
    let event_suffix = if message_existed {
        "updated"
    } else {
        "recorded"
    };
    let event_id = format!(
        "evt_communication_message_{}_{}",
        event_suffix, accepted_event.event_id
    );
    let event = NewEventEnvelope::builder(
        event_id,
        event_name,
        Utc::now(),
        json!({
            "kind": "communication_projection",
            "consumer": COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
        }),
        json!({
            "kind": "communication_message",
            "id": message.message_id,
            "entity_id": message.message_id,
            "message_id": message.message_id,
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
        "accepted_signal_event_id": accepted_event.event_id,
        "accepted_signal_event_type": accepted_event.event_type,
        "projection_kind": event_suffix,
    }))
    .provenance(json!({
        "ownership": "communications_projection",
        "source_event_id": accepted_event.event_id,
    }))
    .causation_id(accepted_event.event_id.clone())
    .correlation_id(
        accepted_event
            .correlation_id
            .clone()
            .unwrap_or_else(|| accepted_event.event_id.clone()),
    )
    .build()?;

    EventStore::new(pool).append_idempotent(&event).await?;
    Ok(())
}

async fn communication_message_exists(
    pool: &PgPool,
    account_id: &str,
    provider_record_id: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM communication_messages
            WHERE account_id = $1
              AND provider_record_id = $2
        )
        "#,
    )
    .bind(account_id.trim())
    .bind(provider_record_id.trim())
    .fetch_one(pool)
    .await
}

async fn raw_record_for_accepted_signal(
    pool: PgPool,
    event: &EventEnvelope,
) -> Result<StoredRawCommunicationRecord, CommunicationSignalProjectionError> {
    let raw_record_id = required_subject_str(&event.subject, "raw_record_id")?;
    CommunicationIngestionStore::new(pool)
        .raw_record(raw_record_id)
        .await?
        .ok_or_else(|| MessageProjectionError::RawRecordNotFound(raw_record_id.to_owned()).into())
}

async fn zulip_target_message(
    pool: PgPool,
    raw_record: &StoredRawCommunicationRecord,
) -> Result<ProviderChannelMessage, CommunicationSignalProjectionError> {
    ensure_canonical_communication_account(&pool, &raw_record.account_id).await?;
    let provider_message_id = required_payload_str(&raw_record.payload, "provider_message_id")?;
    ProviderChannelMessageStore::new(pool)
        .message_by_provider_record_id(
            &raw_record.account_id,
            &provider_message_id,
            &[ZULIP_CHANNEL_KIND],
        )
        .await?
        .ok_or_else(|| {
            ProviderCommunicationMessagePortError::InvalidRequest(format!(
                "zulip target message `{provider_message_id}` is not projected"
            ))
            .into()
        })
}

async fn ensure_canonical_communication_account(
    pool: &PgPool,
    account_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id,
            config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            jsonb_build_object('source_table', 'communication_provider_accounts'),
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id)
        DO UPDATE SET
            provider_kind = EXCLUDED.provider_kind,
            display_name = EXCLUDED.display_name,
            external_account_id = EXCLUDED.external_account_id,
            config = EXCLUDED.config,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(account_id.trim())
    .execute(pool)
    .await?;
    Ok(())
}

async fn projected_message_by_id(
    pool: &PgPool,
    message_id: &str,
) -> Result<Option<ProjectedMessage>, CommunicationSignalProjectionError> {
    let row = sqlx::query(
        r#"
        SELECT
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
            (SELECT s.ai_state FROM communication_ai_states s WHERE s.message_id = communication_messages.message_id) AS ai_state,
            local_state,
            local_state_changed_at,
            local_state_reason,
            is_read,
            read_changed_at,
            read_origin
        FROM communication_messages
        WHERE message_id = $1
        "#,
    )
    .bind(message_id.trim())
    .fetch_optional(pool)
    .await?;

    Ok(row.map(row_to_projected_message).transpose()?)
}

fn merged_zulip_message_metadata(
    current: &Value,
    patch: Value,
) -> Result<Value, CommunicationSignalProjectionError> {
    let Some(current) = current.as_object() else {
        return Err(MessageProjectionError::InvalidMessageMetadata.into());
    };
    let Some(patch) = patch.as_object() else {
        return Err(MessageProjectionError::InvalidMessageMetadata.into());
    };
    let mut merged = current.clone();
    for (key, value) in patch {
        if !value.is_null() {
            merged.insert(key.clone(), value.clone());
        }
    }
    Ok(Value::Object(merged))
}

fn zulip_observed_at(
    raw_record: &StoredRawCommunicationRecord,
    event: &EventEnvelope,
) -> DateTime<Utc> {
    raw_record.occurred_at.unwrap_or(event.occurred_at)
}

fn zulip_content_diff(previous_text: Option<&str>, new_text: &str) -> Value {
    json!({
        "changed": previous_text != Some(new_text),
        "previous_text_length": previous_text.map(|text| text.chars().count()),
        "new_text_length": new_text.chars().count(),
    })
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

fn zulip_message_id(account_id: &str, provider_message_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(provider_message_id.as_bytes());
    format!("message:v1:zulip:{:x}", hasher.finalize())
}

fn zulip_conversation_id(account_id: &str, stream_name: &str, topic: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(stream_name.as_bytes());
    hasher.update(b"\0");
    hasher.update(topic.as_bytes());
    format!("zulip:conversation:{:x}", hasher.finalize())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ZulipMessageTarget {
    subject: String,
    recipients: Vec<String>,
    conversation_id: String,
}

fn zulip_message_target(account_id: &str, payload: &Value) -> ZulipMessageTarget {
    let direct_recipients = zulip_direct_recipient_refs(payload);
    let message_type = optional_payload_str(payload, "message_type").unwrap_or_default();
    if is_zulip_direct_message(&message_type) || !direct_recipients.is_empty() {
        let display_names = zulip_direct_recipient_display_names(payload);
        let recipients = if direct_recipients.is_empty() {
            vec!["Zulip direct".to_owned()]
        } else {
            direct_recipients
        };
        let subject_suffix = if display_names.is_empty() {
            recipients.join(", ")
        } else {
            display_names.join(", ")
        };
        return ZulipMessageTarget {
            subject: if subject_suffix.trim().is_empty() {
                "Direct message".to_owned()
            } else {
                format!("Direct / {subject_suffix}")
            },
            conversation_id: zulip_direct_conversation_id(account_id, &recipients),
            recipients,
        };
    }

    let stream_name = optional_payload_str(payload, "stream_name")
        .or_else(|| optional_payload_str(payload, "stream_id"))
        .unwrap_or_else(|| "Zulip".to_owned());
    let topic = optional_payload_str(payload, "topic").unwrap_or_else(|| "message".to_owned());
    ZulipMessageTarget {
        subject: format!("{stream_name} / {topic}"),
        recipients: vec![stream_name.clone()],
        conversation_id: zulip_conversation_id(account_id, &stream_name, &topic),
    }
}

fn is_zulip_direct_message(message_type: &str) -> bool {
    matches!(message_type.trim(), "private" | "direct")
}

fn zulip_direct_recipient_refs(payload: &Value) -> Vec<String> {
    zulip_direct_recipient_values(
        payload,
        &["email", "full_name", "display_name", "provider_user_id"],
    )
}

fn zulip_direct_recipient_display_names(payload: &Value) -> Vec<String> {
    zulip_direct_recipient_values(
        payload,
        &["full_name", "display_name", "email", "provider_user_id"],
    )
}

fn zulip_direct_recipient_values(payload: &Value, fields: &[&str]) -> Vec<String> {
    let Some(recipients) = payload.get("direct_recipients").and_then(Value::as_array) else {
        return Vec::new();
    };

    recipients
        .iter()
        .filter_map(|recipient| {
            if let Some(value) = recipient
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                return Some(value.to_owned());
            }
            let object = recipient.as_object()?;
            fields.iter().find_map(|field| {
                object
                    .get(*field)
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
            })
        })
        .collect()
}

fn zulip_direct_conversation_id(account_id: &str, recipients: &[String]) -> String {
    let mut recipient_refs = recipients
        .iter()
        .map(|recipient| recipient.trim())
        .filter(|recipient| !recipient.is_empty())
        .collect::<Vec<_>>();
    recipient_refs.sort_unstable();

    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0direct\0");
    for recipient in recipient_refs {
        hasher.update(recipient.as_bytes());
        hasher.update(b"\0");
    }
    format!("zulip:direct_conversation:{:x}", hasher.finalize())
}

fn zulip_reaction_id(
    account_id: &str,
    provider_message_id: &str,
    provider_actor_id: &str,
    reaction: &str,
) -> String {
    stable_zulip_id(
        "message_reaction:v1:zulip",
        &[account_id, provider_message_id, provider_actor_id, reaction],
    )
}

fn zulip_message_version_id(accepted_event_id: &str) -> String {
    stable_zulip_id("message_version:v1:zulip", &[accepted_event_id])
}

fn zulip_message_tombstone_id(accepted_event_id: &str) -> String {
    stable_zulip_id("message_tombstone:v1:zulip", &[accepted_event_id])
}

fn stable_zulip_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.trim().as_bytes());
        hasher.update(b"\0");
    }
    format!("{prefix}:{:x}", hasher.finalize())
}

#[derive(Debug, Error)]
pub enum CommunicationSignalProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Communication(#[from] hermes_communications_postgres::errors::CommunicationIngestionError),

    #[error(transparent)]
    ProviderCommunication(#[from] ProviderCommunicationMessagePortError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error("accepted signal subject is missing `{0}`")]
    MissingSubjectField(&'static str),
}
