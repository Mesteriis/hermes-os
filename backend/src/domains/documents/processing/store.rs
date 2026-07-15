use sqlx::postgres::PgPool;

use super::errors::DocumentProcessingError;
use super::models::{
    DocumentProcessingArtifact, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingStep,
};
use super::rows::{try_row_to_artifact, try_row_to_job};
use super::validation::{validate_non_empty, validate_optional_limit};

#[derive(Clone)]
pub struct DocumentProcessingStore {
    pub(super) pool: PgPool,
}

impl DocumentProcessingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;
        self.ensure_document_exists(&document_id).await?;
        let extract_text_job = self
            .upsert_job(&document_id, DocumentProcessingStep::ExtractText)
            .await?;
        let ocr_job = self
            .upsert_job(&document_id, DocumentProcessingStep::Ocr)
            .await?;

        Ok(vec![extract_text_job, ocr_job])
    }

    pub async fn list_jobs(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let limit = validate_optional_limit(limit)?;

        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            ORDER BY queued_at DESC, job_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_job)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn list_jobs_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingJob>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE document_id = $1
            ORDER BY queued_at
            "#,
        )
        .bind(&document_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_job)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn list_artifacts_for_document(
        &self,
        document_id: &str,
    ) -> Result<Vec<DocumentProcessingArtifact>, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                artifact_id,
                document_id,
                job_id,
                artifact_kind,
                content_sha256,
                text_content,
                storage_kind,
                storage_path,
                metadata,
                created_at
            FROM document_artifacts
            WHERE document_id = $1
            ORDER BY artifact_kind
            "#,
        )
        .bind(&document_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(try_row_to_artifact)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn document_processing(
        &self,
        document_id: &str,
    ) -> Result<DocumentProcessingRecord, DocumentProcessingError> {
        let document_id = validate_non_empty("document_id", document_id)?;
        let Some(_) = self.document_record_by_id(&document_id).await? else {
            return Err(DocumentProcessingError::DocumentNotFound);
        };

        let jobs = self.list_jobs_for_document(&document_id).await?;
        let artifacts = self.list_artifacts_for_document(&document_id).await?;

        Ok(DocumentProcessingRecord {
            document_id,
            jobs,
            artifacts,
        })
    }
}
