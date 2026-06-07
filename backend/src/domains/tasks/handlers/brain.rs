use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::brain::TaskBrainService;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
#[derive(Deserialize)]
pub struct BrainQuery {
    pub q: Option<String>,
}
pub async fn brain(
    State(s): State<AppState>,
    Query(q): Query<BrainQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = TaskBrainService::explain_task(&pool, q.q.as_deref().unwrap_or("")).await?;
    Ok(Json(result))
}
