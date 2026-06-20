use std::path::Path;

use serde_json::{Value, json};
use sqlx::PgPool;

use crate::domains::communications::storage::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanStatus, AttachmentSafetyScanner,
    ImportedCommunicationAttachment, LocalMailBlobStore, MailAttachmentDisposition,
    MailStorageStore, NewMailAttachment, NewMailBlob, NoopAttachmentSafetyScanner,
};
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::integrations::telegram::runtime::{
    TelegramMediaDownloadRequest, TelegramMediaDownloadResponse, TelegramMediaSendRequest,
    TelegramMediaSendType,
};
use crate::integrations::telegram::tdjson::TelegramTdlibFileSnapshot;
use crate::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;

pub(crate) async fn persist_downloaded_media(
    telegram_store: &TelegramStore,
    pool: PgPool,
    request: &TelegramMediaDownloadRequest,
    file: &TelegramTdlibFileSnapshot,
    blob_root: &Path,
) -> Result<TelegramMediaDownloadResponse, TelegramError> {
    let mut response = TelegramMediaDownloadResponse {
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
        TelegramError::TdlibRuntime(
            "TDLib reported a completed download without a local file path".to_owned(),
        )
    })?;
    let bytes = tokio::fs::read(local_path).await.map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to read downloaded Telegram file `{local_path}`: {error}"
        ))
    })?;
    let blob_store = LocalMailBlobStore::new(blob_root);
    let local_blob = blob_store.put_blob(&bytes).await.map_err(|error| {
        TelegramError::TdlibRuntime(format!("failed to store Telegram media blob: {error}"))
    })?;
    let mail_store = MailStorageStore::new(pool);
    let content_type = request.content_type();
    let stored_blob = mail_store
        .upsert_blob(&NewMailBlob::from_local_blob(&local_blob).content_type(content_type.clone()))
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("failed to record Telegram media blob: {error}"))
        })?;
    let scanner = NoopAttachmentSafetyScanner;
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
            TelegramError::TdlibRuntime(format!("Telegram media scan failed: {error}"))
        })?;
    let anchor = telegram_store
        .attachment_anchor_for_message(
            &request.account_id,
            &request.provider_chat_id,
            &request.provider_message_id,
        )
        .await?;
    let mut attachment = NewMailAttachment::new(
        anchor.message_id,
        anchor.raw_record_id,
        stored_blob.blob_id.clone(),
        provider_attachment_id,
        content_type,
        local_blob.size_bytes,
        local_blob.sha256.clone(),
    )
    .disposition(MailAttachmentDisposition::Attachment)
    .scan_report(scan_report);
    if let Some(filename) = filename {
        attachment = attachment.filename(filename);
    }
    let stored_attachment = mail_store
        .upsert_attachment(&attachment)
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!(
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
            .await
            .map_err(|error| TelegramError::MediaStorage(error.to_string()))?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "attachment import `{attachment_id}` was not found"
                ))
            })?
    } else {
        let blob_id = blob_id.as_deref().expect("blob_id checked above");
        if let Some(imported) = mail_store
            .imported_attachment_by_blob_id(blob_id)
            .await
            .map_err(|error| TelegramError::MediaStorage(error.to_string()))?
        {
            imported
        } else {
            let blob = mail_store
                .blob_by_id(blob_id)
                .await
                .map_err(|error| TelegramError::MediaStorage(error.to_string()))?
                .ok_or_else(|| {
                    TelegramError::InvalidRequest(format!("blob `{blob_id}` was not found"))
                })?;
            ImportedCommunicationAttachment {
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
                scan_status: AttachmentSafetyScanStatus::NotScanned,
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
