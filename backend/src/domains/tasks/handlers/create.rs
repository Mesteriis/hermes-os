use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::api::{NewTask, Task, TaskStore};
use axum::Json;
use axum::extract::State;
pub(crate) async fn create(
    State(s): State<AppState>,
    Json(r): Json<NewTask>,
) -> Result<Json<Task>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(TaskStore::new(pool).create(&r).await?))
}
