use chrono::Utc;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::domains::mail::background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::domains::mail::storage::MailStorageStore;
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::runtime::{TelegramMediaSendRequest, TelegramMediaSendType};
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

pub(super) async fn media_send_request(
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
) -> Result<TelegramMediaSendRequest, TelegramError> {
    let media_type =
        TelegramMediaSendType::try_from(payload_string(command, "media_type")?.as_str())?;
    let attachment_id = payload_optional_string(command, "attachment_id");
    let blob_id = payload_optional_string(command, "blob_id");
    if attachment_id.is_none() && blob_id.is_none() {
        return Err(TelegramError::InvalidRequest(
            "send_media command requires attachment_id or blob_id".to_owned(),
        ));
    }

    let mail_store = MailStorageStore::new(pool.clone());
    let imported = if let Some(attachment_id) = attachment_id.as_deref() {
        mail_store
            .imported_attachment_by_id(attachment_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "attachment import `{attachment_id}` was not found"
                ))
            })?
    } else {
        let blob_id = blob_id.as_deref().expect("blob_id checked above");
        if let Some(imported) = mail_store.imported_attachment_by_blob_id(blob_id).await? {
            imported
        } else {
            let blob = mail_store.blob_by_id(blob_id).await?.ok_or_else(|| {
                TelegramError::InvalidRequest(format!("blob `{blob_id}` was not found"))
            })?;
            crate::domains::mail::storage::ImportedCommunicationAttachment {
                attachment_id: format!("blob:{blob_id}"),
                account_id: Some(command.account_id.clone()),
                channel_kind: Some("telegram".to_owned()),
                blob_id: blob.blob_id,
                filename: payload_optional_string(command, "filename"),
                content_type: blob
                    .content_type
                    .unwrap_or_else(|| "application/octet-stream".to_owned()),
                size_bytes: blob.size_bytes,
                sha256: blob.sha256,
                source_kind: "blob_reuse".to_owned(),
                imported_by: "telegram-outbox-worker".to_owned(),
                scan_status: crate::domains::mail::storage::AttachmentSafetyScanStatus::NotScanned,
                scan_engine: None,
                scan_checked_at: None,
                scan_summary: None,
                scan_metadata: json!({}),
                metadata: json!({}),
                storage_kind: blob.storage_kind,
                storage_path: blob.storage_path,
                created_at: blob.created_at,
                updated_at: blob.created_at,
            }
        }
    };

    if imported.storage_kind != "local_fs" {
        return Err(TelegramError::InvalidRequest(
            "send_media requires a local filesystem blob".to_owned(),
        ));
    }
    if imported.scan_status.as_str() == "malicious" {
        return Err(TelegramError::InvalidRequest(
            "send_media rejected a malicious attachment import".to_owned(),
        ));
    }
    let local_path = std::path::Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
        .join(&imported.storage_path)
        .to_string_lossy()
        .into_owned();

    Ok(TelegramMediaSendRequest {
        command_id: command.command_id.clone(),
        provider_chat_id: command.provider_chat_id.clone(),
        media_type,
        local_path,
        caption: payload_optional_string(command, "caption"),
        filename: imported.filename,
    })
}

pub(super) async fn emit_media_upload_event(
    event_bus: &EventBus,
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    event_type: &str,
    extra_payload: Value,
) {
    let now = Utc::now();
    let mut payload = json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "provider_chat_id": command.provider_chat_id,
        "attachment_id": payload_optional_string(command, "attachment_id"),
        "blob_id": payload_optional_string(command, "blob_id"),
        "media_type": payload_optional_string(command, "media_type"),
        "caption_present": payload_optional_string(command, "caption").is_some(),
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    let event = NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_media_upload"}),
    )
    .payload(payload)
    .build();

    let Ok(event) = event else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "command executor: failed to append media upload event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) fn media_upload_progress_payload(
    command: &TelegramProviderWriteCommand,
    phase: &str,
    detail: &str,
) -> Value {
    let mut provider_state = command.provider_state.clone();
    if let Some(provider_state_obj) = provider_state.as_object_mut() {
        provider_state_obj.insert("upload_phase".to_owned(), Value::String(phase.to_owned()));
        provider_state_obj.insert(
            "progress_detail".to_owned(),
            Value::String(detail.to_owned()),
        );
    }
    json!({
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "provider_state": provider_state,
        "reconciliation_status": command.reconciliation_status,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
        "progress_phase": phase,
        "progress_detail": detail,
    })
}

fn payload_string(
    command: &TelegramProviderWriteCommand,
    key: &str,
) -> Result<String, TelegramError> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{} command missing `{key}`",
                command.command_kind
            ))
        })
}

fn payload_optional_string(command: &TelegramProviderWriteCommand, key: &str) -> Option<String> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::media_upload_progress_payload;
    use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;

    fn sample_command() -> TelegramProviderWriteCommand {
        TelegramProviderWriteCommand {
            command_id: "cmd-1".to_owned(),
            account_id: "account-1".to_owned(),
            command_kind: "send_media".to_owned(),
            idempotency_key: "idem-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            provider_message_id: None,
            target_ref: json!({}),
            payload: json!({"attachment_id": "att-1", "blob_id": "blob-1"}),
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "confirmed".to_owned(),
            status: "executing".to_owned(),
            retry_count: 1,
            max_retries: 3,
            last_error: None,
            result_payload: json!({}),
            audit_metadata: json!({}),
            actor_id: "hermes-frontend".to_owned(),
            happened_at: Utc::now(),
            next_attempt_at: None,
            last_attempt_at: None,
            locked_at: None,
            locked_by: None,
            provider_observed_at: None,
            provider_state: json!({"dispatch": "claimed"}),
            reconciliation_status: "not_observed".to_owned(),
            reconciled_at: None,
            dead_lettered_at: None,
            completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn media_upload_progress_payload_carries_phase_detail_in_provider_state() {
        let payload = media_upload_progress_payload(
            &sample_command(),
            "dispatching_to_provider",
            "Uploading local media to Telegram",
        );

        assert_eq!(payload["status"], "executing");
        assert_eq!(payload["progress_phase"], "dispatching_to_provider");
        assert_eq!(
            payload["progress_detail"],
            "Uploading local media to Telegram"
        );
        assert_eq!(payload["provider_state"]["dispatch"], "claimed");
        assert_eq!(
            payload["provider_state"]["upload_phase"],
            "dispatching_to_provider"
        );
        assert_eq!(
            payload["provider_state"]["progress_detail"],
            "Uploading local media to Telegram"
        );
    }
}
