//! WhatsApp WebView media command handling and scanned attachment resolution.

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::app::api_support::stores::{domain_stores::*, integration_stores::*};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::communications::storage::scanner::AttachmentSafetyScanStatus;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppMediaDownloadRequest, WhatsAppMediaUploadRequest, WhatsAppProviderCommandResponse,
    WhatsAppVoiceNoteSendRequest,
};
use crate::platform::events::bus::whatsapp_event_types;

use super::{
    optional_string, publish_whatsapp_command_event, publish_whatsapp_media_event, required_string,
};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppMediaUploadApiRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) media_type: String,
    pub(crate) caption: Option<String>,
    pub(crate) filename: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppMediaDownloadApiRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) provider_attachment_id: Option<String>,
    pub(crate) provider_media_id: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WhatsAppValidatedMediaUploadRequest {
    command_id: Option<String>,
    idempotency_key: Option<String>,
    account_id: String,
    provider_chat_id: String,
    attachment_id: Option<String>,
    blob_id: Option<String>,
    media_type: String,
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

pub(crate) async fn post_whatsapp_media_upload(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppMediaUploadApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_media_upload_request(request)?;
    let attachment =
        resolve_whatsapp_upload_attachment(&communication_storage_store(&state)?, &request).await?;
    let runtime_request = WhatsAppMediaUploadRequest {
        command_id: request.command_id.clone(),
        idempotency_key: request.idempotency_key.clone().unwrap_or_else(|| {
            whatsapp_media_upload_idempotency_key(&request, &attachment.blob_id)
        }),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        attachment_id: attachment.attachment_id.clone(),
        blob_id: attachment.blob_id.clone(),
        media_type: request.media_type.clone(),
        caption: request.caption.clone(),
        filename: request
            .filename
            .clone()
            .or_else(|| attachment.filename.clone()),
        content_type: attachment.content_type.clone(),
        size_bytes: attachment.size_bytes,
        sha256: attachment.sha256.clone(),
        scan_status: attachment.scan_status.clone(),
    };
    let response = whatsapp_provider_runtime_service(&state)?
        .request_media_upload(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &runtime_request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    let upload_attachment_id = runtime_request.attachment_id.clone();
    let upload_blob_id = runtime_request.blob_id.clone();
    let upload_media_type = runtime_request.media_type.clone();
    let upload_filename = runtime_request.filename.clone();
    let upload_content_type = runtime_request.content_type.clone();
    let upload_scan_status = runtime_request.scan_status.clone();
    let response_command_id = response.command_id.clone();
    let response_account_id = response.account_id.clone();
    let response_provider_chat_id = response.provider_chat_id.clone();
    let response_command_kind = response.command_kind.clone();
    publish_whatsapp_media_event(
        &state,
        whatsapp_event_types::MEDIA_UPLOAD_REQUESTED,
        &response.command_id,
        json!({
            "command_id": response_command_id,
            "account_id": response_account_id,
            "provider_chat_id": response_provider_chat_id,
            "command_kind": response_command_kind,
            "blob_id": upload_blob_id,
            "attachment_id": upload_attachment_id,
            "media_type": upload_media_type,
            "filename": upload_filename,
            "content_type": upload_content_type,
            "scan_status": upload_scan_status,
            "status": "requested",
        }),
    )
    .await?;
    if response.status == "blocked" {
        publish_whatsapp_media_event(
            &state,
            whatsapp_event_types::MEDIA_UPLOAD_FAILED,
            &response.command_id,
            json!({
                "command_id": response.command_id,
                "account_id": response.account_id,
                "provider_chat_id": response.provider_chat_id,
                "command_kind": response.command_kind,
                "blob_id": runtime_request.blob_id,
                "attachment_id": runtime_request.attachment_id,
                "media_type": runtime_request.media_type,
                "status": "failed",
                "error": response.last_error,
                "runtime_blockers": response.runtime_blockers,
            }),
        )
        .await?;
    }
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_media_download(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppMediaDownloadApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let runtime_request = validate_whatsapp_media_download_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_media_download(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &runtime_request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    let download_provider_message_id = response.provider_message_id.clone();
    let download_provider_attachment_id = runtime_request.provider_attachment_id.clone();
    let download_provider_media_id = runtime_request.provider_media_id.clone();
    let download_filename = runtime_request.filename.clone();
    let download_content_type = runtime_request.content_type.clone();
    let response_command_id = response.command_id.clone();
    let response_account_id = response.account_id.clone();
    let response_provider_chat_id = response.provider_chat_id.clone();
    let response_command_kind = response.command_kind.clone();
    publish_whatsapp_media_event(
        &state,
        whatsapp_event_types::MEDIA_DOWNLOAD_REQUESTED,
        &response.command_id,
        json!({
            "command_id": response_command_id,
            "account_id": response_account_id,
            "provider_chat_id": response_provider_chat_id,
            "provider_message_id": download_provider_message_id,
            "command_kind": response_command_kind,
            "provider_attachment_id": download_provider_attachment_id,
            "provider_media_id": download_provider_media_id,
            "filename": download_filename,
            "content_type": download_content_type,
            "status": "requested",
        }),
    )
    .await?;
    if response.status == "blocked" {
        publish_whatsapp_media_event(
            &state,
            whatsapp_event_types::MEDIA_DOWNLOAD_FAILED,
            &response.command_id,
            json!({
                "command_id": response.command_id,
                "account_id": response.account_id,
                "provider_chat_id": response.provider_chat_id,
                "provider_message_id": response.provider_message_id,
                "command_kind": response.command_kind,
                "provider_attachment_id": runtime_request.provider_attachment_id,
                "provider_media_id": runtime_request.provider_media_id,
                "status": "failed",
                "error": response.last_error,
                "runtime_blockers": response.runtime_blockers,
            }),
        )
        .await?;
    }
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_voice_note_send(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppMediaUploadApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_media_upload_request(request)?;
    let attachment =
        resolve_whatsapp_upload_attachment(&communication_storage_store(&state)?, &request).await?;
    let runtime_request = WhatsAppVoiceNoteSendRequest {
        command_id: request.command_id.clone(),
        idempotency_key: request.idempotency_key.clone().unwrap_or_else(|| {
            whatsapp_media_upload_idempotency_key(&request, &attachment.blob_id)
        }),
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        attachment_id: attachment.attachment_id,
        blob_id: attachment.blob_id,
        filename: request.filename.or(attachment.filename),
        content_type: attachment.content_type,
        size_bytes: attachment.size_bytes,
        sha256: attachment.sha256,
        scan_status: attachment.scan_status,
    };
    let response = whatsapp_provider_runtime_service(&state)?
        .request_send_voice_note(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &runtime_request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

fn validate_whatsapp_media_upload_request(
    request: WhatsAppMediaUploadApiRequest,
) -> Result<WhatsAppValidatedMediaUploadRequest, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    let provider_chat_id = required_string("provider_chat_id", &request.provider_chat_id)?;
    let media_type = required_string("media_type", &request.media_type)?;
    let attachment_id =
        optional_string("attachment_id", request.attachment_id)?.ok_or_else(|| {
            WhatsappWebError::InvalidRequest(
                "attachment_id is required so WhatsApp media can be sent only after a clean scan"
                    .to_owned(),
            )
        })?;
    let blob_id = optional_string("blob_id", request.blob_id)?;

    Ok(WhatsAppValidatedMediaUploadRequest {
        command_id: request.command_id,
        idempotency_key: optional_string("idempotency_key", request.idempotency_key)?,
        account_id,
        provider_chat_id,
        attachment_id: Some(attachment_id),
        blob_id,
        media_type,
        caption: optional_string("caption", request.caption)?,
        filename: optional_string("filename", request.filename)?,
    })
}

fn validate_whatsapp_media_download_request(
    request: WhatsAppMediaDownloadApiRequest,
) -> Result<WhatsAppMediaDownloadRequest, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    let provider_chat_id = required_string("provider_chat_id", &request.provider_chat_id)?;
    let provider_message_id = required_string("provider_message_id", &request.provider_message_id)?;
    let provider_attachment_id =
        optional_string("provider_attachment_id", request.provider_attachment_id)?;
    let provider_media_id = optional_string("provider_media_id", request.provider_media_id)?;
    if provider_attachment_id.is_none() && provider_media_id.is_none() {
        return Err(WhatsappWebError::InvalidRequest(
            "provider_attachment_id or provider_media_id is required".to_owned(),
        )
        .into());
    }

    Ok(WhatsAppMediaDownloadRequest {
        command_id: request.command_id,
        idempotency_key: request.idempotency_key.unwrap_or_else(|| {
            let mut hasher = Sha256::new();
            hasher.update(account_id.as_bytes());
            hasher.update(b"\0");
            hasher.update(provider_chat_id.as_bytes());
            hasher.update(b"\0");
            hasher.update(provider_message_id.as_bytes());
            hasher.update(b"\0");
            if let Some(value) = provider_attachment_id.as_deref() {
                hasher.update(value.as_bytes());
            }
            hasher.update(b"\0");
            if let Some(value) = provider_media_id.as_deref() {
                hasher.update(value.as_bytes());
            }
            format!("whatsapp:media-download:{:x}", hasher.finalize())
        }),
        account_id,
        provider_chat_id,
        provider_message_id,
        provider_attachment_id,
        provider_media_id,
        filename: optional_string("filename", request.filename)?,
        content_type: optional_string("content_type", request.content_type)?,
    })
}
async fn resolve_whatsapp_upload_attachment(
    storage: &crate::domains::communications::storage::store::CommunicationStorageStore,
    request: &WhatsAppValidatedMediaUploadRequest,
) -> Result<UploadAttachmentRef, ApiError> {
    if let Some(attachment_id) = request.attachment_id.as_deref() {
        let imported = storage
            .imported_attachment_by_id(attachment_id)
            .await
            .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "attachment import `{attachment_id}` was not found"
                ))
            })?;
        if let Some(import_account_id) = imported.account_id.as_deref()
            && import_account_id != request.account_id
        {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "attachment import `{attachment_id}` belongs to a different account"
            ))
            .into());
        }
        if let Some(channel_kind) = imported.channel_kind.as_deref()
            && !matches!(channel_kind, "whatsapp" | "whatsapp_web")
        {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "attachment import `{attachment_id}` is scoped to `{channel_kind}`, not WhatsApp"
            ))
            .into());
        }
        if let Some(blob_id) = request.blob_id.as_deref()
            && blob_id != imported.blob_id
        {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "blob_id `{blob_id}` does not match attachment import `{attachment_id}`"
            ))
            .into());
        }
        if imported.storage_kind != "local_fs" {
            return Err(WhatsappWebError::InvalidRequest(
                "WhatsApp media upload requires a local filesystem blob".to_owned(),
            )
            .into());
        }
        if imported.scan_status != AttachmentSafetyScanStatus::Clean {
            return Err(WhatsappWebError::InvalidRequest(
                "WhatsApp media upload requires a clean attachment import".to_owned(),
            )
            .into());
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

    unreachable!("validate_whatsapp_media_upload_request requires attachment_id")
}

fn whatsapp_media_upload_idempotency_key(
    request: &WhatsAppValidatedMediaUploadRequest,
    resolved_blob_id: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(request.account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(request.provider_chat_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(request.media_type.as_bytes());
    hasher.update(b"\0");
    hasher.update(resolved_blob_id.as_bytes());
    hasher.update(b"\0");
    if let Some(caption) = request.caption.as_deref() {
        hasher.update(caption.as_bytes());
    }
    format!("whatsapp:media-upload:{:x}", hasher.finalize())
}
