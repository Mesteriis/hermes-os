use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::CommunicationStorageError;
use super::models::{
    CommunicationAttachmentDisposition, ImportedCommunicationAttachment,
    StoredCommunicationAttachment, StoredCommunicationAttachmentWithBlob, StoredCommunicationBlob,
};
use super::scanner::AttachmentSafetyScanStatus;

pub(crate) fn row_to_mail_blob(
    row: PgRow,
) -> Result<StoredCommunicationBlob, CommunicationStorageError> {
    Ok(StoredCommunicationBlob {
        blob_id: row.try_get("blob_id")?,
        storage_kind: row.try_get("storage_kind")?,
        storage_path: row.try_get("storage_path")?,
        sha256: row.try_get("sha256")?,
        size_bytes: row.try_get("size_bytes")?,
        content_type: row.try_get("content_type")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(crate) fn row_to_mail_attachment(
    row: PgRow,
) -> Result<StoredCommunicationAttachment, CommunicationStorageError> {
    let disposition: String = row.try_get("disposition")?;
    let scan_status: String = row.try_get("scan_status")?;

    Ok(StoredCommunicationAttachment {
        attachment_id: row.try_get("attachment_id")?,
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        blob_id: row.try_get("blob_id")?,
        provider_attachment_id: row.try_get("provider_attachment_id")?,
        filename: row.try_get("filename")?,
        content_type: row.try_get("content_type")?,
        size_bytes: row.try_get("size_bytes")?,
        sha256: row.try_get("sha256")?,
        disposition: CommunicationAttachmentDisposition::try_from(disposition.as_str())?,
        scan_status: AttachmentSafetyScanStatus::try_from(scan_status.as_str())?,
        scan_engine: row.try_get("scan_engine")?,
        scan_checked_at: row.try_get("scan_checked_at")?,
        scan_summary: row.try_get("scan_summary")?,
        scan_metadata: row.try_get("scan_metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(crate) fn row_to_mail_attachment_with_blob(
    row: PgRow,
) -> Result<StoredCommunicationAttachmentWithBlob, CommunicationStorageError> {
    let storage_kind: String = row.try_get("blob_storage_kind")?;
    let storage_path: String = row.try_get("blob_storage_path")?;
    Ok(StoredCommunicationAttachmentWithBlob {
        attachment: row_to_mail_attachment(row)?,
        storage_kind,
        storage_path,
    })
}

pub(crate) fn row_to_imported_attachment(
    row: PgRow,
) -> Result<ImportedCommunicationAttachment, CommunicationStorageError> {
    let scan_status: String = row.try_get("scan_status")?;
    Ok(ImportedCommunicationAttachment {
        attachment_id: row.try_get("attachment_id")?,
        account_id: row.try_get("account_id")?,
        channel_kind: row.try_get("channel_kind")?,
        blob_id: row.try_get("blob_id")?,
        filename: row.try_get("filename")?,
        content_type: row.try_get("content_type")?,
        size_bytes: row.try_get("size_bytes")?,
        sha256: row.try_get("sha256")?,
        source_kind: row.try_get("source_kind")?,
        imported_by: row.try_get("imported_by")?,
        scan_status: AttachmentSafetyScanStatus::try_from(scan_status.as_str())?,
        scan_engine: row.try_get("scan_engine")?,
        scan_checked_at: row.try_get("scan_checked_at")?,
        scan_summary: row.try_get("scan_summary")?,
        scan_metadata: row.try_get("scan_metadata")?,
        metadata: row.try_get("metadata")?,
        storage_kind: row.try_get("blob_storage_kind")?,
        storage_path: row.try_get("blob_storage_path")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
