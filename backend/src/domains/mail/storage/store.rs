use sqlx::postgres::PgPool;

use super::errors::MailStorageError;
use super::ids::{mail_attachment_id, mail_blob_id};
use super::models::{
    NewMailAttachment, NewMailBlob, StoredMailAttachment, StoredMailAttachmentWithBlob,
    StoredMailBlob,
};
use super::rows::{row_to_mail_attachment, row_to_mail_attachment_with_blob, row_to_mail_blob};
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct MailStorageStore {
    pub(crate) pool: PgPool,
}

impl MailStorageStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_blob(
        &self,
        blob: &NewMailBlob,
    ) -> Result<StoredMailBlob, MailStorageError> {
        let blob = blob.validate()?;
        let blob_id = mail_blob_id(&blob.sha256);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_mail_blobs (
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (sha256)
            DO UPDATE SET
                content_type = COALESCE(communication_mail_blobs.content_type, EXCLUDED.content_type)
            RETURNING
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type,
                created_at
            "#,
        )
        .bind(&blob_id)
        .bind(&blob.storage_kind)
        .bind(&blob.storage_path)
        .bind(&blob.sha256)
        .bind(blob.size_bytes)
        .bind(&blob.content_type)
        .fetch_one(&self.pool)
        .await?;

        row_to_mail_blob(row)
    }

    pub async fn upsert_attachment(
        &self,
        attachment: &NewMailAttachment,
    ) -> Result<StoredMailAttachment, MailStorageError> {
        let attachment = attachment.validate()?;
        let attachment_id =
            mail_attachment_id(&attachment.message_id, &attachment.provider_attachment_id);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_attachments (
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
            ON CONFLICT (message_id, provider_attachment_id)
            DO UPDATE SET
                raw_record_id = EXCLUDED.raw_record_id,
                blob_id = EXCLUDED.blob_id,
                filename = EXCLUDED.filename,
                content_type = EXCLUDED.content_type,
                size_bytes = EXCLUDED.size_bytes,
                sha256 = EXCLUDED.sha256,
                disposition = EXCLUDED.disposition,
                scan_status = EXCLUDED.scan_status,
                scan_engine = EXCLUDED.scan_engine,
                scan_checked_at = EXCLUDED.scan_checked_at,
                scan_summary = EXCLUDED.scan_summary,
                scan_metadata = EXCLUDED.scan_metadata,
                updated_at = now()
            RETURNING
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&attachment_id)
        .bind(&attachment.message_id)
        .bind(&attachment.raw_record_id)
        .bind(&attachment.blob_id)
        .bind(&attachment.provider_attachment_id)
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size_bytes)
        .bind(&attachment.sha256)
        .bind(attachment.disposition.as_str())
        .bind(attachment.scan_report.status.as_str())
        .bind(&attachment.scan_report.engine)
        .bind(attachment.scan_report.checked_at)
        .bind(&attachment.scan_report.summary)
        .bind(&attachment.scan_report.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_mail_attachment(row)
    }

    pub async fn attachments_for_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<StoredMailAttachmentWithBlob>, MailStorageError> {
        let message_id = validate_non_empty("message_id", message_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
                a.blob_id,
                a.provider_attachment_id,
                a.filename,
                a.content_type,
                a.size_bytes,
                a.sha256,
                a.disposition,
                a.scan_status,
                a.scan_engine,
                a.scan_checked_at,
                a.scan_summary,
                a.scan_metadata,
                a.created_at,
                a.updated_at,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path
            FROM communication_attachments a
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE a.message_id = $1
            ORDER BY a.created_at ASC, a.attachment_id ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_mail_attachment_with_blob)
            .collect()
    }

    pub async fn attachment_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<Option<StoredMailAttachmentWithBlob>, MailStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
                a.blob_id,
                a.provider_attachment_id,
                a.filename,
                a.content_type,
                a.size_bytes,
                a.sha256,
                a.disposition,
                a.scan_status,
                a.scan_engine,
                a.scan_checked_at,
                a.scan_summary,
                a.scan_metadata,
                a.created_at,
                a.updated_at,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path
            FROM communication_attachments a
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE a.attachment_id = $1
            "#,
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_mail_attachment_with_blob).transpose()
    }
}
