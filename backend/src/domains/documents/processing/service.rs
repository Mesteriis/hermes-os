use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::{
    DocumentProcessingError, DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult,
    DocumentProcessingStore,
};

#[derive(Clone)]
pub struct DocumentProcessingCommandService {
    pool: PgPool,
}

impl DocumentProcessingCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn retry_failed_job_manual(
        &self,
        command: &DocumentProcessingRetryCommand,
    ) -> Result<DocumentProcessingRetryCommandResult, DocumentProcessingCommandServiceError> {
        let result = DocumentProcessingStore::new(self.pool.clone())
            .retry_failed_job(command)
            .await?;

        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "DOCUMENT_PROCESSING_JOB_STATUS",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "job_id": result.job_id,
                        "status": serde_json::to_value(result.status).unwrap_or(Value::Null),
                        "event_id": result.event_id,
                        "operation": "document_processing_retry",
                    }),
                    format!("document-processing://jobs/{}/retry", result.job_id),
                )
                .provenance(json!({
                    "captured_by": "documents.processing_service.retry_failed_job_manual",
                    "operation": "retry_failed_job_manual",
                })),
            )
            .await?;

        DocumentProcessingStore::new(self.pool.clone())
            .retry_failed_job_with_observation(command, Some(&observation.observation_id))
            .await?;

        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum DocumentProcessingCommandServiceError {
    #[error(transparent)]
    DocumentProcessing(#[from] DocumentProcessingError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
