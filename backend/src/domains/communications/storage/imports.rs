use super::errors::CommunicationStorageError;
use super::ids::communication_attachment_import_id;
use super::models::{ImportedCommunicationAttachment, NewCommunicationAttachmentImport};
use super::rows::{row_to_imported_attachment, row_to_mail_blob};
use super::store::CommunicationStorageStore;
use super::validation::validate_non_empty;
use crate::domains::communications::evidence::link_mail_entity_in_transaction;

impl CommunicationStorageStore {
    pub async fn upsert_imported_attachment(
        &self,
        import: &NewCommunicationAttachmentImport,
    ) -> Result<ImportedCommunicationAttachment, CommunicationStorageError> {
        self.upsert_imported_attachment_with_observation(import, None, "attachment_import", None)
            .await
    }

    pub async fn upsert_imported_attachment_with_observation(
        &self,
        import: &NewCommunicationAttachmentImport,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<ImportedCommunicationAttachment, CommunicationStorageError> {
        let import = import.validate()?;
        let mut transaction = self.pool.begin().await?;
        sqlx::query(imported_attachment_upsert_sql())
            .bind(&import.attachment_id)
            .bind(&import.account_id)
            .bind(&import.channel_kind)
            .bind(&import.blob_id)
            .bind(&import.filename)
            .bind(&import.content_type)
            .bind(import.size_bytes)
            .bind(&import.sha256)
            .bind(&import.source_kind)
            .bind(&import.imported_by)
            .bind(import.scan_report.status.as_str())
            .bind(&import.scan_report.engine)
            .bind(import.scan_report.checked_at)
            .bind(&import.scan_report.summary)
            .bind(&import.scan_report.metadata)
            .bind(&import.metadata)
            .execute(&mut *transaction)
            .await?;

        let sql = imported_attachment_select_sql("i.attachment_id = $1");
        let row = sqlx::query(&sql)
            .bind(&import.attachment_id)
            .fetch_optional(&mut *transaction)
            .await?;
        let imported = row
            .map(row_to_imported_attachment)
            .transpose()?
            .ok_or_else(|| CommunicationStorageError::Sqlx(sqlx::Error::RowNotFound))?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_mail_entity_in_transaction(
                &mut transaction,
                observation_id,
                "attachment_import",
                imported.attachment_id.clone(),
                relationship_kind,
                serde_json::json!({
                    "blob_id": imported.blob_id,
                    "scan_status": imported.scan_status.as_str(),
                    "content_type": imported.content_type,
                    "sha256": imported.sha256,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(imported)
    }

    pub async fn imported_attachment_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<Option<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let sql = imported_attachment_select_sql("i.attachment_id = $1");
        let row = sqlx::query(&sql)
            .bind(attachment_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_imported_attachment).transpose()
    }

    pub async fn imported_attachment_by_blob_id(
        &self,
        blob_id: &str,
    ) -> Result<Option<ImportedCommunicationAttachment>, CommunicationStorageError> {
        let blob_id = validate_non_empty("blob_id", blob_id)?;
        let sql = imported_attachment_select_sql("i.blob_id = $1");
        let row = sqlx::query(&sql)
            .bind(blob_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_imported_attachment).transpose()
    }

    pub async fn blob_by_id(
        &self,
        blob_id: &str,
    ) -> Result<Option<super::models::StoredCommunicationBlob>, CommunicationStorageError> {
        let blob_id = validate_non_empty("blob_id", blob_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type,
                created_at
            FROM communication_mail_blobs
            WHERE blob_id = $1
            "#,
        )
        .bind(blob_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_mail_blob).transpose()
    }
}

pub fn new_communication_attachment_import_id(seed: &str) -> String {
    communication_attachment_import_id(seed)
}

fn imported_attachment_upsert_sql() -> &'static str {
    r#"
    INSERT INTO communication_attachment_imports (
        attachment_id,
        account_id,
        channel_kind,
        blob_id,
        filename,
        content_type,
        size_bytes,
        sha256,
        source_kind,
        imported_by,
        scan_status,
        scan_engine,
        scan_checked_at,
        scan_summary,
        scan_metadata,
        metadata,
        updated_at
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, now())
    ON CONFLICT (attachment_id)
    DO UPDATE SET
        account_id = EXCLUDED.account_id,
        channel_kind = EXCLUDED.channel_kind,
        blob_id = EXCLUDED.blob_id,
        filename = EXCLUDED.filename,
        content_type = EXCLUDED.content_type,
        size_bytes = EXCLUDED.size_bytes,
        sha256 = EXCLUDED.sha256,
        source_kind = EXCLUDED.source_kind,
        imported_by = EXCLUDED.imported_by,
        scan_status = EXCLUDED.scan_status,
        scan_engine = EXCLUDED.scan_engine,
        scan_checked_at = EXCLUDED.scan_checked_at,
        scan_summary = EXCLUDED.scan_summary,
        scan_metadata = EXCLUDED.scan_metadata,
        metadata = EXCLUDED.metadata,
        updated_at = now()
    "#
}

fn imported_attachment_select_sql(predicate: &str) -> String {
    format!(
        r#"
        SELECT
            i.attachment_id,
            i.account_id,
            i.channel_kind,
            i.blob_id,
            i.filename,
            i.content_type,
            i.size_bytes,
            i.sha256,
            i.source_kind,
            i.imported_by,
            i.scan_status,
            i.scan_engine,
            i.scan_checked_at,
            i.scan_summary,
            i.scan_metadata,
            i.metadata,
            b.storage_kind AS blob_storage_kind,
            b.storage_path AS blob_storage_path,
            i.created_at,
            i.updated_at
        FROM communication_attachment_imports i
        JOIN communication_mail_blobs b ON b.blob_id = i.blob_id
        WHERE {predicate}
        ORDER BY i.created_at DESC
        LIMIT 1
        "#
    )
}
