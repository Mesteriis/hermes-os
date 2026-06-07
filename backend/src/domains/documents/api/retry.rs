use super::dto::{RetryRequest, RetryResponse};
use crate::app::handlers::{ApiError, AppState};
use crate::domains::documents::processing::{
    DocumentProcessingRetryCommand, DocumentProcessingStore,
};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};
use axum::Json;
use axum::extract::{Path, State};

const ACTOR: &str = "hermes-frontend";

pub(crate) async fn retry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(_req): Json<RetryRequest>,
) -> Result<Json<RetryResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    ApiAuditLog::new(pool.clone())
        .record(&NewApiAuditRecord::document_processing_job_retry(
            ACTOR, &id,
        ))
        .await?;
    let result = DocumentProcessingStore::new(pool)
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("retry-{}", id),
            actor_id: ACTOR.into(),
            job_id: id,
        })
        .await?;
    Ok(Json(RetryResponse {
        job_id: result.job_id,
        status: format!("{:?}", result.status),
    }))
}
