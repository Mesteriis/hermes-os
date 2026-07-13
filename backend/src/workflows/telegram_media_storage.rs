use crate::domains::communications::storage::port::{CommunicationAttachmentPort, LocalBlobPort};
use std::path::Path;

use serde_json::Value;
use sqlx::PgPool;
use thiserror::Error;

use crate::domains::communications::storage::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanner, CommunicationAttachmentDisposition,
    HeuristicAttachmentSafetyScanner, NewCommunicationAttachment, NewCommunicationBlob,
};
use crate::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramMediaDownloadData {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) tdlib_file_id: i64,
    pub(crate) provider_attachment_id: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: Option<String>,
}

impl TelegramMediaDownloadData {
    pub(crate) fn provider_attachment_id(&self) -> String {
        self.provider_attachment_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("tdlib-file:{}", self.tdlib_file_id))
    }

    pub(crate) fn content_type(&self) -> String {
        self.content_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "application/octet-stream".to_owned())
    }

    pub(crate) fn filename(&self) -> Option<String> {
        self.filename
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramDownloadedFileData {
    pub(crate) file_id: i64,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) expected_size_bytes: Option<i64>,
    pub(crate) local_path: Option<String>,
    pub(crate) is_downloading_active: bool,
    pub(crate) is_downloading_completed: bool,
    pub(crate) downloaded_size_bytes: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramAttachmentAnchor {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramMediaDownloadProjection {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) tdlib_file_id: i64,
    pub(crate) local_path: Option<String>,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) expected_size_bytes: Option<i64>,
    pub(crate) downloaded_size_bytes: Option<i64>,
    pub(crate) is_downloading_active: bool,
    pub(crate) is_downloading_completed: bool,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) scan_status: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramProviderMediaCommand {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) command_kind: String,
    pub(crate) provider_chat_id: String,
    pub(crate) payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramPreparedMediaSendRequest {
    pub(crate) command_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) media_type: String,
    pub(crate) local_path: String,
    pub(crate) caption: Option<String>,
    pub(crate) filename: Option<String>,
}

#[derive(Debug, Error)]
pub(crate) enum TelegramMediaStorageError {
    #[error("invalid Telegram media request: {0}")]
    InvalidRequest(String),

    #[error("Telegram media runtime error: {0}")]
    Runtime(String),

    #[error("Telegram media storage error: {0}")]
    Storage(String),
}

pub(crate) async fn persist_downloaded_media(
    pool: PgPool,
    request: &TelegramMediaDownloadData,
    file: &TelegramDownloadedFileData,
    anchor: Option<TelegramAttachmentAnchor>,
    blob_root: &Path,
) -> Result<TelegramMediaDownloadProjection, TelegramMediaStorageError> {
    let mut response = TelegramMediaDownloadProjection {
        account_id: request.account_id.trim().to_owned(),
        provider_chat_id: request.provider_chat_id.trim().to_owned(),
        provider_message_id: request.provider_message_id.trim().to_owned(),
        runtime_kind: "tdlib_qr_authorized".to_owned(),
        status: if file.is_downloading_completed {
            "downloaded".to_owned()
        } else if file.is_downloading_active {
            "downloading".to_owned()
        } else {
            "remote".to_owned()
        },
        tdlib_file_id: file.file_id,
        local_path: file.local_path.clone(),
        size_bytes: file.size_bytes,
        expected_size_bytes: file.expected_size_bytes,
        downloaded_size_bytes: file.downloaded_size_bytes,
        is_downloading_active: file.is_downloading_active,
        is_downloading_completed: file.is_downloading_completed,
        attachment_id: None,
        blob_id: None,
        scan_status: None,
    };

    if !file.is_downloading_completed {
        return Ok(response);
    }

    let local_path = file.local_path.as_deref().ok_or_else(|| {
        TelegramMediaStorageError::Runtime(
            "TDLib reported a completed download without a local file path".to_owned(),
        )
    })?;
    let bytes = tokio::fs::read(local_path).await.map_err(|error| {
        TelegramMediaStorageError::Runtime(format!(
            "failed to read downloaded Telegram file `{local_path}`: {error}"
        ))
    })?;
    let blob_store = LocalBlobPort::new(blob_root);
    let local_blob = blob_store.put_blob(&bytes).await.map_err(|error| {
        TelegramMediaStorageError::Storage(format!("failed to store Telegram media blob: {error}"))
    })?;
    let mail_store = CommunicationAttachmentPort::new(pool);
    let content_type = request.content_type();
    let stored_blob = mail_store
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&local_blob).content_type(content_type.clone()),
        )
        .await
        .map_err(|error| {
            TelegramMediaStorageError::Storage(format!(
                "failed to record Telegram media blob: {error}"
            ))
        })?;
    let scanner = HeuristicAttachmentSafetyScanner;
    let provider_attachment_id = request.provider_attachment_id();
    let filename = request.filename();
    let scan_report = scanner
        .scan(&AttachmentSafetyScanRequest {
            provider_attachment_id: &provider_attachment_id,
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &local_blob.sha256,
            storage_kind: &local_blob.storage_kind,
            storage_path: &local_blob.storage_path,
            bytes: &bytes,
        })
        .map_err(|error| {
            TelegramMediaStorageError::Storage(format!("Telegram media scan failed: {error}"))
        })?;
    let anchor = anchor.ok_or_else(|| {
        TelegramMediaStorageError::InvalidRequest(
            "completed Telegram media download requires a communication message anchor".to_owned(),
        )
    })?;
    let mut attachment = NewCommunicationAttachment::new(
        anchor.message_id,
        anchor.raw_record_id,
        stored_blob.blob_id.clone(),
        provider_attachment_id,
        content_type,
        local_blob.size_bytes,
        local_blob.sha256.clone(),
    )
    .disposition(CommunicationAttachmentDisposition::Attachment)
    .scan_report(scan_report);
    if let Some(filename) = filename {
        attachment = attachment.filename(filename);
    }
    let stored_attachment = mail_store
        .upsert_attachment(&attachment)
        .await
        .map_err(|error| {
            TelegramMediaStorageError::Storage(format!(
                "failed to record Telegram media attachment: {error}"
            ))
        })?;

    response.attachment_id = Some(stored_attachment.attachment_id);
    response.blob_id = Some(stored_blob.blob_id);
    response.scan_status = Some(stored_attachment.scan_status.as_str().to_owned());
    Ok(response)
}

pub(crate) async fn media_send_request(
    pool: &PgPool,
    command: &TelegramProviderMediaCommand,
) -> Result<TelegramPreparedMediaSendRequest, TelegramMediaStorageError> {
    let media_type = payload_string(command, "media_type")?;
    validate_media_type(&media_type)?;
    let attachment_id = payload_optional_string(command, "attachment_id").ok_or_else(|| {
        TelegramMediaStorageError::InvalidRequest(
            "send_media command requires attachment_id so a clean scan can be enforced".to_owned(),
        )
    })?;
    let blob_id = payload_optional_string(command, "blob_id");

    let mail_store = CommunicationAttachmentPort::new(pool.clone());
    let imported = mail_store
        .imported_attachment_by_id(&attachment_id)
        .await
        .map_err(|error| TelegramMediaStorageError::Storage(error.to_string()))?
        .ok_or_else(|| {
            TelegramMediaStorageError::InvalidRequest(format!(
                "attachment import `{attachment_id}` was not found"
            ))
        })?;

    if let Some(blob_id) = blob_id.as_deref()
        && blob_id != imported.blob_id
    {
        return Err(TelegramMediaStorageError::InvalidRequest(format!(
            "blob_id `{blob_id}` does not match attachment import `{attachment_id}`"
        )));
    }

    if imported.storage_kind != "local_fs" {
        return Err(TelegramMediaStorageError::InvalidRequest(
            "send_media requires a local filesystem blob".to_owned(),
        ));
    }
    if imported.scan_status.as_str() != "clean" {
        return Err(TelegramMediaStorageError::InvalidRequest(
            "send_media requires a clean attachment import".to_owned(),
        ));
    }
    let local_path = std::path::Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
        .join(&imported.storage_path)
        .to_string_lossy()
        .into_owned();

    Ok(TelegramPreparedMediaSendRequest {
        command_id: command.command_id.clone(),
        provider_chat_id: command.provider_chat_id.clone(),
        media_type,
        local_path,
        caption: payload_optional_string(command, "caption"),
        filename: imported.filename,
    })
}

fn payload_string(
    command: &TelegramProviderMediaCommand,
    key: &str,
) -> Result<String, TelegramMediaStorageError> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramMediaStorageError::InvalidRequest(format!(
                "{} command missing `{key}`",
                command.command_kind
            ))
        })
}

fn payload_optional_string(command: &TelegramProviderMediaCommand, key: &str) -> Option<String> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn validate_media_type(media_type: &str) -> Result<(), TelegramMediaStorageError> {
    match media_type.trim() {
        "photo" | "video" | "document" | "audio" | "voice" | "voice_note" | "sticker"
        | "animation" | "gif" => Ok(()),
        other => Err(TelegramMediaStorageError::InvalidRequest(format!(
            "unsupported Telegram media upload type `{other}`"
        ))),
    }
}
