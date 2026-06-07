use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::brain::TaskBrainService;
use axum::Json;
use axum::extract::State;
pub(crate) async fn daily_brief(
    State(s): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = TaskBrainService::daily_brief(&pool).await?;
    Ok(Json(brief))
}
