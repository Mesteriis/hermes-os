use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::DocumentProcessingError;
use super::models::{
    DocumentArtifactKind, DocumentProcessingArtifact, DocumentProcessingJob,
    DocumentProcessingStatus, DocumentProcessingStep,
};

pub(super) fn try_row_to_job(row: PgRow) -> Result<DocumentProcessingJob, DocumentProcessingError> {
    let step = DocumentProcessingStep::parse(row.try_get::<String, _>("step")?.as_str())?;
    let status = DocumentProcessingStatus::parse(row.try_get::<String, _>("status")?.as_str())?;

    Ok(DocumentProcessingJob {
        job_id: row.try_get("job_id")?,
        document_id: row.try_get("document_id")?,
        step,
        status,
        attempts: row.try_get("attempts")?,
        max_attempts: row.try_get("max_attempts")?,
        last_error_summary: row.try_get("last_error_summary")?,
        queued_at: row.try_get("queued_at")?,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn try_row_to_artifact(
    row: PgRow,
) -> Result<DocumentProcessingArtifact, DocumentProcessingError> {
    let artifact_kind =
        DocumentArtifactKind::parse(row.try_get::<String, _>("artifact_kind")?.as_str())?;

    Ok(DocumentProcessingArtifact {
        artifact_id: row.try_get("artifact_id")?,
        document_id: row.try_get("document_id")?,
        job_id: row.try_get("job_id")?,
        artifact_kind,
        content_sha256: row.try_get("content_sha256")?,
        text_content: row.try_get("text_content")?,
        storage_kind: row.try_get("storage_kind")?,
        storage_path: row.try_get("storage_path")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}
