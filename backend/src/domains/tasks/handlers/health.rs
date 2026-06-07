use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::health::TaskWatchtowerService;
use axum::Json;
use axum::extract::State;
pub async fn health(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(TaskWatchtowerService::stale_tasks(&pool, 30).await?))
}
