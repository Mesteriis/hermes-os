use axum::Json;
use axum::extract::State;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::helpers::{
    AUDIT_ACTOR_ID, publish_telegram_event, telegram_message_snapshot_payload,
    telegram_runtime_event_bridge_context, telegram_secret_store,
};
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{
    api_audit_log, communication_provider_account_store,
    communication_provider_secret_binding_store, mail_storage_store, telegram_store,
};
use crate::domains::mail::storage::AttachmentSafetyScanStatus;
use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::TelegramCommandKind;
use crate::integrations::telegram::client::{TelegramError, ensure_telegram_account_active};
use crate::integrations::telegram::runtime::{
    TelegramMediaDownloadContext, TelegramMediaDownloadRequest, TelegramMediaDownloadResponse,
    TelegramMediaSendType,
};
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::NewEventEnvelope;
use crate::platform::events::bus::telegram_event_types;
use crate::vault::CommunicationProviderAccountStore;

fn build_event(
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": subject_id, "kind": "telegram_message"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

fn build_upload_event(
    event_type: &str,
    account_id: &str,
    command_id: &str,
    provider_chat_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    let mut event_payload = json!({
        "command_id": command_id,
        "account_id": account_id,
        "provider_chat_id": provider_chat_id,
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (event_payload.as_object_mut(), payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
        payload_obj.insert("payload".to_owned(), payload);
    }
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": command_id, "kind": "telegram_command"}),
    )
    .payload(event_payload)
    .build()
    .expect("event envelope must be valid")
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TelegramMediaUploadRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) media_type: String,
    pub(crate) caption: Option<String>,
    pub(crate) filename: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TelegramMediaUploadResponse {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: String,
    pub(crate) media_type: String,
    pub(crate) status: String,
    pub(crate) reconciliation_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedMediaUploadRequest {
    command_id: String,
    account_id: String,
    provider_chat_id: String,
    attachment_id: Option<String>,
    blob_id: Option<String>,
    media_type: TelegramMediaSendType,
    caption: Option<String>,
    filename: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UploadAttachmentRef {
    attachment_id: Option<String>,
    blob_id: String,
    content_type: String,
    filename: Option<String>,
    size_bytes: i64,
    sha256: String,
    scan_status: String,
}

pub(crate) async fn post_telegram_media_upload(
    State(state): State<AppState>,
    Json(request): Json<TelegramMediaUploadRequest>,
) -> Result<Json<TelegramMediaUploadResponse>, ApiError> {
    let request = validate_media_upload_request(request)?;
    let pool = state
        .database
        .pool()
        .cloned()
        .ok_or(ApiError::DatabaseNotConfigured)?;
    let provider_account_store = communication_provider_account_store(&state)?;
    let account = provider_account_store
        .get(&request.account_id)
        .await?
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram account `{}` was not found",
                request.account_id
            ))
        })?;
    if !account.provider_kind.is_telegram() {
        return Err(TelegramError::InvalidRequest(format!(
            "account `{}` is not a Telegram provider account",
            account.account_id
        ))
        .into());
    }
    ensure_telegram_account_active(&account)?;
    let runtime_kind = account
        .config
        .get("runtime")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    if runtime_kind != "tdlib_qr_authorized" {
        return Err(TelegramError::InvalidRequest(format!(
            "Telegram media upload requires a tdlib_qr_authorized account; `{}` uses `{runtime_kind}`",
            account.account_id
        ))
        .into());
    }

    let mail_store = mail_storage_store(&state)?;
    let attachment = resolve_upload_attachment(&mail_store, &request).await?;
    let audit_metadata = json!({
        "capability": "telegram.media.upload",
        "action_class": "provider_write",
        "confirmation_decision": "explicit_user_confirmation",
        "attachment_id": &attachment.attachment_id,
        "blob_id": &attachment.blob_id,
        "media_type": request.media_type.as_str(),
        "content_type": &attachment.content_type,
        "size_bytes": attachment.size_bytes,
        "sha256": &attachment.sha256,
        "scan_status": &attachment.scan_status,
    });
    let idempotency_key = media_upload_idempotency_key(&request, &attachment.blob_id);
    if let Some(existing) =
        lifecycle::find_command_by_idempotency(&pool, &request.account_id, &idempotency_key).await?
    {
        return Ok(Json(media_upload_response(&existing)));
    }
    let command = lifecycle::insert_command(
        &pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::SendMedia.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        AUDIT_ACTOR_ID,
        json!({
            "attachment_id": attachment.attachment_id.clone(),
            "blob_id": attachment.blob_id.clone(),
            "media_type": request.media_type.as_str(),
            "caption": request.caption.clone(),
            "filename": request.filename.clone().or(attachment.filename.clone()),
            "content_type": attachment.content_type.clone(),
            "size_bytes": attachment.size_bytes,
            "sha256": attachment.sha256.clone(),
        }),
        json!({
            "provider_chat_id": request.provider_chat_id,
            "attachment_id": request.attachment_id,
            "blob_id": request.blob_id,
        }),
        audit_metadata.clone(),
    )
    .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_media_upload(
            AUDIT_ACTOR_ID,
            &command.command_id,
            &command.account_id,
            &command.provider_chat_id,
            command
                .payload
                .get("attachment_id")
                .and_then(serde_json::Value::as_str),
            command
                .payload
                .get("blob_id")
                .and_then(serde_json::Value::as_str),
            command
                .payload
                .get("media_type")
                .and_then(serde_json::Value::as_str),
        ))
        .await?;

    publish_telegram_event(
        &state,
        build_upload_event(
            telegram_event_types::MEDIA_UPLOAD_STARTED,
            &command.account_id,
            &command.command_id,
            &command.provider_chat_id,
            json!({
                "command_kind": command.command_kind,
                "idempotency_key": command.idempotency_key,
                "payload": command.payload,
                "target_ref": command.target_ref,
                "capability_state": command.capability_state,
                "action_class": command.action_class,
                "confirmation_decision": command.confirmation_decision,
                "status": &command.status,
                "retry_count": command.retry_count,
                "max_retries": command.max_retries,
                "last_error": command.last_error,
                "result_payload": command.result_payload,
                "audit_metadata": command.audit_metadata,
                "actor_id": command.actor_id,
                "happened_at": command.happened_at,
                "next_attempt_at": command.next_attempt_at,
                "last_attempt_at": command.last_attempt_at,
                "provider_observed_at": command.provider_observed_at,
                "provider_state": command.provider_state,
                "reconciliation_status": command.reconciliation_status,
                "reconciled_at": command.reconciled_at,
                "dead_lettered_at": command.dead_lettered_at,
                "completed_at": command.completed_at,
                "created_at": command.created_at,
                "updated_at": command.updated_at,
                "attachment_id": command.payload.get("attachment_id").cloned(),
                "blob_id": command.payload.get("blob_id").cloned(),
                "media_type": command.payload.get("media_type").cloned(),
                "filename": command.payload.get("filename").cloned(),
            }),
        ),
    )
    .await?;
    publish_telegram_event(
        &state,
        build_upload_event(
            telegram_event_types::COMMAND_STATUS_CHANGED,
            &command.account_id,
            &command.command_id,
            &command.provider_chat_id,
            json!({"status": &command.status, "source": "media_upload_api"}),
        ),
    )
    .await?;

    Ok(Json(media_upload_response(&command)))
}

fn media_upload_response(
    command: &crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand,
) -> TelegramMediaUploadResponse {
    TelegramMediaUploadResponse {
        command_id: command.command_id.clone(),
        account_id: command.account_id.clone(),
        provider_chat_id: command.provider_chat_id.clone(),
        attachment_id: command
            .payload
            .get("attachment_id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        blob_id: command
            .payload
            .get("blob_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        media_type: command
            .payload
            .get("media_type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        status: command.status.clone(),
        reconciliation_status: command.reconciliation_status.clone(),
    }
}

pub(crate) async fn post_telegram_media_download(
    State(state): State<AppState>,
    Json(request): Json<TelegramMediaDownloadRequest>,
) -> Result<Json<TelegramMediaDownloadResponse>, ApiError> {
    let started = build_event(
        telegram_event_types::MEDIA_DOWNLOAD_STARTED,
        &request.account_id,
        &request.provider_message_id,
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": &request.provider_message_id,
            "tdlib_file_id": request.tdlib_file_id,
            "provider_attachment_id": request.provider_attachment_id(),
            "download_state": "requested",
        }),
    );
    publish_telegram_event(&state, started).await?;

    let secret_store = telegram_secret_store(&state)?;
    let provider_account_store = communication_provider_account_store(&state)?;
    let provider_secret_binding_store = communication_provider_secret_binding_store(&state)?;
    let telegram_store = telegram_store(&state)?;
    let mail_store = mail_storage_store(&state)?;
    let response = match state
        .telegram_runtime
        .download_media(
            TelegramMediaDownloadContext {
                provider_account_store: &provider_account_store,
                provider_secret_binding_store: &provider_secret_binding_store,
                telegram_store: &telegram_store,
                mail_store: &mail_store,
                secret_store: &secret_store,
                secret_resolver: &state.vault,
                config: &state.config,
                event_bridge: Some(telegram_runtime_event_bridge_context(&state)),
            },
            &request,
        )
        .await
    {
        Ok(response) => response,
        Err(error) => {
            let failed = build_event(
                telegram_event_types::MEDIA_DOWNLOAD_FAILED,
                &request.account_id,
                &request.provider_message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "tdlib_file_id": request.tdlib_file_id,
                    "provider_attachment_id": request.provider_attachment_id(),
                    "download_state": "failed",
                    "error": error.to_string(),
                }),
            );
            publish_telegram_event(&state, failed).await?;
            return Err(error.into());
        }
    };

    if response.is_downloading_completed {
        let attachment_anchor = telegram_store
            .attachment_anchor_for_message(
                &request.account_id,
                &request.provider_chat_id,
                &request.provider_message_id,
            )
            .await?;
        telegram_store
            .update_message_attachment_download_state(
                &attachment_anchor.message_id,
                &request.provider_attachment_id(),
                response.tdlib_file_id,
                &response.status,
                response.local_path.as_deref(),
                response.size_bytes,
                &request.content_type(),
                request.filename().as_deref(),
            )
            .await?;
        let event = build_event(
            telegram_event_types::MEDIA_DOWNLOADED,
            &request.account_id,
            &attachment_anchor.message_id,
            telegram_message_snapshot_payload(
                &telegram_store,
                &attachment_anchor.message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "tdlib_file_id": response.tdlib_file_id,
                    "download_state": &response.status,
                    "local_path": response.local_path.clone(),
                    "attachment_id": response.attachment_id.clone(),
                    "blob_id": response.blob_id.clone(),
                    "scan_status": response.scan_status.clone(),
                }),
            )
            .await?,
        );
        publish_telegram_event(&state, event).await?;
    } else {
        let progress = build_event(
            telegram_event_types::MEDIA_DOWNLOAD_PROGRESS,
            &request.account_id,
            &request.provider_message_id,
            json!({
                "provider_chat_id": &request.provider_chat_id,
                "provider_message_id": &request.provider_message_id,
                "tdlib_file_id": response.tdlib_file_id,
                "provider_attachment_id": request.provider_attachment_id(),
                "download_state": &response.status,
                "expected_size_bytes": response.expected_size_bytes,
                "downloaded_size_bytes": response.downloaded_size_bytes,
                "is_downloading_active": response.is_downloading_active,
                "is_downloading_completed": response.is_downloading_completed,
            }),
        );
        publish_telegram_event(&state, progress).await?;
    }

    Ok(Json(response))
}

fn validate_media_upload_request(
    request: TelegramMediaUploadRequest,
) -> Result<ValidatedMediaUploadRequest, TelegramError> {
    let account_id = required_string("account_id", &request.account_id)?;
    let provider_chat_id = required_string("provider_chat_id", &request.provider_chat_id)?;
    let media_type = TelegramMediaSendType::try_from(request.media_type.as_str())?;
    let command_id = match request.command_id {
        Some(command_id) => required_string("command_id", &command_id)?,
        None => lifecycle::new_command_id(),
    };
    let attachment_id = optional_string("attachment_id", request.attachment_id)?;
    let blob_id = optional_string("blob_id", request.blob_id)?;
    if attachment_id.is_none() && blob_id.is_none() {
        return Err(TelegramError::InvalidRequest(
            "attachment_id or blob_id is required".to_owned(),
        ));
    }
    let caption = optional_string("caption", request.caption)?;
    let filename = optional_string("filename", request.filename)?;

    Ok(ValidatedMediaUploadRequest {
        command_id,
        account_id,
        provider_chat_id,
        attachment_id,
        blob_id,
        media_type,
        caption,
        filename,
    })
}

async fn resolve_upload_attachment(
    mail_store: &crate::domains::mail::storage::MailStorageStore,
    request: &ValidatedMediaUploadRequest,
) -> Result<UploadAttachmentRef, TelegramError> {
    if let Some(attachment_id) = request.attachment_id.as_deref() {
        let imported = mail_store
            .imported_attachment_by_id(attachment_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "attachment import `{attachment_id}` was not found"
                ))
            })?;
        if let Some(import_account_id) = imported.account_id.as_deref()
            && import_account_id != request.account_id
        {
            return Err(TelegramError::InvalidRequest(format!(
                "attachment import `{attachment_id}` belongs to a different account"
            )));
        }
        if let Some(channel_kind) = imported.channel_kind.as_deref()
            && !matches!(channel_kind, "telegram" | "telegram_user" | "telegram_bot")
        {
            return Err(TelegramError::InvalidRequest(format!(
                "attachment import `{attachment_id}` is scoped to `{channel_kind}`, not Telegram"
            )));
        }
        if let Some(blob_id) = request.blob_id.as_deref()
            && blob_id != imported.blob_id
        {
            return Err(TelegramError::InvalidRequest(format!(
                "blob_id `{blob_id}` does not match attachment import `{attachment_id}`"
            )));
        }
        if imported.storage_kind != "local_fs" {
            return Err(TelegramError::InvalidRequest(
                "Telegram media upload requires a local filesystem blob".to_owned(),
            ));
        }
        if imported.scan_status == AttachmentSafetyScanStatus::Malicious {
            return Err(TelegramError::InvalidRequest(
                "Telegram media upload rejected a malicious attachment import".to_owned(),
            ));
        }

        return Ok(UploadAttachmentRef {
            attachment_id: Some(imported.attachment_id),
            blob_id: imported.blob_id,
            content_type: imported.content_type,
            filename: imported.filename,
            size_bytes: imported.size_bytes,
            sha256: imported.sha256,
            scan_status: imported.scan_status.as_str().to_owned(),
        });
    }

    let blob_id = request
        .blob_id
        .as_deref()
        .expect("blob_id presence checked by validate_media_upload_request");
    let blob = mail_store
        .blob_by_id(blob_id)
        .await?
        .ok_or_else(|| TelegramError::InvalidRequest(format!("blob `{blob_id}` was not found")))?;
    if blob.storage_kind != "local_fs" {
        return Err(TelegramError::InvalidRequest(
            "Telegram media upload requires a local filesystem blob".to_owned(),
        ));
    }

    Ok(UploadAttachmentRef {
        attachment_id: None,
        blob_id: blob.blob_id,
        content_type: blob
            .content_type
            .unwrap_or_else(|| "application/octet-stream".to_owned()),
        filename: request.filename.clone(),
        size_bytes: blob.size_bytes,
        sha256: blob.sha256,
        scan_status: AttachmentSafetyScanStatus::NotScanned.as_str().to_owned(),
    })
}

fn media_upload_idempotency_key(
    request: &ValidatedMediaUploadRequest,
    resolved_blob_id: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(request.account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(request.provider_chat_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(request.media_type.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(resolved_blob_id.as_bytes());
    hasher.update(b"\0");
    if let Some(caption) = request.caption.as_deref() {
        hasher.update(caption.as_bytes());
    }
    format!("telegram:media-upload:{:x}", hasher.finalize())
}

fn required_string(field: &'static str, value: &str) -> Result<String, TelegramError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(value.to_owned())
}

fn optional_string(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, TelegramError> {
    value
        .map(|value| required_string(field, &value))
        .transpose()
}
