use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::api::{Task, TaskStore, TaskUpdate};
use axum::Json;
use axum::extract::{Path, State};
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(r): Json<TaskUpdate>,
) -> Result<Json<Task>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(TaskStore::new(pool).update(&id, &r).await?))
}
