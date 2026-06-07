use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::brain::TaskBrainService;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}
pub async fn search(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let results = TaskBrainService::search_tasks(&pool, &q.q).await?;
    Ok(Json(results))
}
