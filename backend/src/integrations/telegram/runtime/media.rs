use std::path::Path;

use crate::domains::mail::storage::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanner, LocalMailBlobStore,
    MailAttachmentDisposition, MailStorageStore, NewMailAttachment, NewMailBlob,
    NoopAttachmentSafetyScanner,
};
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::integrations::telegram::tdjson::TelegramTdlibFileSnapshot;

use super::models::{TelegramMediaDownloadRequest, TelegramMediaDownloadResponse};

pub(super) async fn persist_downloaded_media(
    telegram_store: &TelegramStore,
    mail_store: &MailStorageStore,
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
