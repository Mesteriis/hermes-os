use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::tasks::health::TaskWatchtowerService;

use super::support::database_pool;

#[derive(Deserialize)]
pub(crate) struct WatchtowerQuery {
    days: Option<i64>,
}

pub(crate) async fn get_task_watchtower(
    State(state): State<AppState>,
    Query(q): Query<WatchtowerQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let days = q.days.unwrap_or(14);
    let overdue = TaskWatchtowerService::overdue(&pool)
        .await
        .map_err(ApiError::from)?;
    let stale = TaskWatchtowerService::stale_tasks(&pool, days)
        .await
        .map_err(ApiError::from)?;
    let no_ctx = TaskWatchtowerService::without_context(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        json!({"overdue": overdue, "stale": stale, "without_context": no_ctx}),
    ))
}

pub(crate) async fn get_task_health(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let wl = TaskWatchtowerService::workload(&pool)
        .await
        .map_err(ApiError::from)?;
    let ct = TaskWatchtowerService::cycle_time(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"workload": wl, "cycle_time": ct})))
}

pub(crate) async fn get_task_analytics(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let _pool = database_pool(&state)?;
    Ok(Json(
        json!({"analytics": "available via /tasks/health and /tasks/watchtower"}),
    ))
}
