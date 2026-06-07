use super::dto::{JobsQuery, JobsResponse};
use crate::app::handlers::{ApiError, AppState};
use crate::domains::documents::processing::DocumentProcessingStore;
use axum::Json;
use axum::extract::{Query, State};

pub(crate) async fn jobs(
    State(state): State<AppState>,
    Query(q): Query<JobsQuery>,
) -> Result<Json<JobsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(JobsResponse {
        items: DocumentProcessingStore::new(pool)
            .list_jobs(q.limit)
            .await?,
    }))
}
