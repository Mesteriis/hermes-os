use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use serde::Serialize;
use serde_json::Value;

use crate::app::api_support::{
    communication_blob_store, communication_storage_store, telegram_provider_runtime_service,
    telegram_runtime_use_case_context,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{TelegramError, TelegramMediaDownloadRequest};
use crate::application::telegram_runtime;
use crate::domains::communications::storage::NewCommunicationBlob;

const MAX_CHAT_AVATAR_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TelegramChatAvatarSyncResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) status: String,
    pub(crate) content_type: Option<String>,
    pub(crate) size_bytes: Option<i64>,
}

pub(crate) async fn post_telegram_chat_avatar_sync(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
) -> Result<Json<TelegramChatAvatarSyncResponse>, ApiError> {
    let service = telegram_provider_runtime_service(&state)?;
    let chat = service
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| TelegramError::InvalidRequest("Telegram chat was not found".to_owned()))?;
    let avatar = chat_avatar_reference(&chat.metadata)?;
    if let Some(local) = local_avatar_for_reference(&chat.metadata, &avatar) {
        return Ok(Json(TelegramChatAvatarSyncResponse {
            telegram_chat_id: chat.telegram_chat_id,
            status: "available".to_owned(),
            content_type: local.content_type,
            size_bytes: local.size_bytes,
        }));
    }

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_runtime::download_media(
        &runtime_context,
        &TelegramMediaDownloadRequest {
            account_id: chat.account_id.clone(),
            provider_chat_id: chat.provider_chat_id.clone(),
            // The runtime file operation is provider-neutral; this synthetic locator is never
            // persisted as a message and prevents an avatar from becoming a message attachment.
            provider_message_id: format!("chat-avatar:{}", chat.telegram_chat_id),
            tdlib_file_id: avatar.tdlib_file_id,
            provider_attachment_id: None,
            filename: None,
            content_type: None,
            priority: Some(4),
        },
    )
    .await?;
    if !response.is_downloading_completed {
        return Ok(Json(TelegramChatAvatarSyncResponse {
            telegram_chat_id: chat.telegram_chat_id,
            status: response.status,
            content_type: None,
            size_bytes: response.size_bytes,
        }));
    }

    let local_path = response.local_path.as_deref().ok_or_else(|| {
        TelegramError::TdlibRuntime(
            "TDLib completed a chat avatar download without a local file path".to_owned(),
        )
    })?;
    let bytes = tokio::fs::read(local_path).await.map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to read downloaded Telegram chat avatar: {error}"
        ))
    })?;
    if bytes.len() > MAX_CHAT_AVATAR_BYTES {
        return Err(TelegramError::InvalidRequest(
            "Telegram chat avatar exceeds the local size limit".to_owned(),
        )
        .into());
    }
    let content_type = chat_avatar_content_type(&bytes).ok_or_else(|| {
        TelegramError::InvalidRequest(
            "Telegram chat avatar is not a supported raster image".to_owned(),
        )
    })?;
    let local_blob = communication_blob_store().put_blob(&bytes).await?;
    let stored_blob = communication_storage_store(&state)?
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type(content_type))
        .await?;
    service
        .apply_local_telegram_chat_avatar(
            &chat.telegram_chat_id,
            avatar.tdlib_file_id,
            avatar.remote_unique_id.as_deref(),
            &stored_blob.blob_id,
            content_type,
            stored_blob.size_bytes,
            &stored_blob.sha256,
        )
        .await?;

    Ok(Json(TelegramChatAvatarSyncResponse {
        telegram_chat_id: chat.telegram_chat_id,
        status: "available".to_owned(),
        content_type: stored_blob.content_type,
        size_bytes: Some(stored_blob.size_bytes),
    }))
}

pub(crate) async fn get_telegram_chat_avatar(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
) -> Result<Response, ApiError> {
    let service = telegram_provider_runtime_service(&state)?;
    let chat = service
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let avatar = chat_avatar_reference(&chat.metadata)?;
    let local = local_avatar_for_reference(&chat.metadata, &avatar).ok_or(ApiError::NotFound)?;
    let blob = communication_storage_store(&state)?
        .blob_by_id(&local.blob_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let content_type = blob
        .content_type
        .as_deref()
        .filter(|value| supported_chat_avatar_content_type(value))
        .ok_or(ApiError::NotFound)?;
    let bytes = communication_blob_store()
        .read_blob(&blob.storage_path)
        .await?;
    if bytes.len() > MAX_CHAT_AVATAR_BYTES || chat_avatar_content_type(&bytes) != Some(content_type)
    {
        return Err(ApiError::NotFound);
    }

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        content_type
            .parse::<HeaderValue>()
            .map_err(|_| ApiError::NotFound)?,
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    Ok(response)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TelegramChatAvatarReference {
    tdlib_file_id: i64,
    remote_unique_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalTelegramChatAvatar {
    blob_id: String,
    content_type: Option<String>,
    size_bytes: Option<i64>,
}

fn chat_avatar_reference(metadata: &Value) -> Result<TelegramChatAvatarReference, ApiError> {
    let avatar = metadata
        .get("avatar")
        .and_then(Value::as_object)
        .ok_or(ApiError::NotFound)?;
    let tdlib_file_id = avatar
        .get("tdlib_file_id")
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)
        .ok_or(ApiError::NotFound)?;
    let remote_unique_id = avatar
        .get("remote_unique_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    Ok(TelegramChatAvatarReference {
        tdlib_file_id,
        remote_unique_id,
    })
}

fn local_avatar_for_reference(
    metadata: &Value,
    reference: &TelegramChatAvatarReference,
) -> Option<LocalTelegramChatAvatar> {
    let local = metadata.get("avatar_local")?.as_object()?;
    let local_file_id = local.get("tdlib_file_id")?.as_i64()?;
    if local_file_id != reference.tdlib_file_id {
        return None;
    }
    let local_remote_unique_id = local.get("remote_unique_id").and_then(Value::as_str);
    if reference
        .remote_unique_id
        .as_deref()
        .is_some_and(|remote_unique_id| local_remote_unique_id != Some(remote_unique_id))
    {
        return None;
    }
    Some(LocalTelegramChatAvatar {
        blob_id: local.get("blob_id")?.as_str()?.to_owned(),
        content_type: local
            .get("content_type")
            .and_then(Value::as_str)
            .map(str::to_owned),
        size_bytes: local.get("size_bytes").and_then(Value::as_i64),
    })
}

fn chat_avatar_content_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    None
}

fn supported_chat_avatar_content_type(content_type: &str) -> bool {
    matches!(content_type, "image/jpeg" | "image/png" | "image/webp")
}

#[cfg(test)]
mod tests {
    use super::{
        TelegramChatAvatarReference, chat_avatar_content_type, local_avatar_for_reference,
    };
    use serde_json::json;

    #[test]
    fn recognizes_only_supported_raster_avatar_types() {
        assert_eq!(
            chat_avatar_content_type(&[0xff, 0xd8, 0xff, 0x00]),
            Some("image/jpeg")
        );
        assert_eq!(
            chat_avatar_content_type(b"\x89PNG\r\n\x1a\nrest"),
            Some("image/png")
        );
        assert_eq!(
            chat_avatar_content_type(b"RIFFxxxxWEBPrest"),
            Some("image/webp")
        );
        assert_eq!(chat_avatar_content_type(b"<svg />"), None);
    }

    #[test]
    fn rejects_local_avatar_when_tdlib_reference_changes() {
        let metadata = json!({
            "avatar_local": {
                "tdlib_file_id": 7,
                "remote_unique_id": "old",
                "blob_id": "blob_1",
                "content_type": "image/jpeg"
            }
        });
        let reference = TelegramChatAvatarReference {
            tdlib_file_id: 8,
            remote_unique_id: Some("new".to_owned()),
        };

        assert!(local_avatar_for_reference(&metadata, &reference).is_none());
    }
}
