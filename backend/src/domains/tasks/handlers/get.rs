use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::api::{Task, TaskStore};
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    TaskStore::new(pool)
        .get(&id)
        .await?
        .ok_or(ApiError::NotFound)
        .map(Json)
}
