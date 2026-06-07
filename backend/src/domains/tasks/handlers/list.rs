use super::dto::{ListQuery, TaskList};
use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::api::{TaskListQuery, TaskStore};
use axum::Json;
use axum::extract::{Query, State};
pub(crate) async fn list(
    State(s): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<TaskList>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let query = TaskListQuery {
        limit: q.limit,
        status: None,
        project_id: None,
        source_type: None,
    };
    Ok(Json(TaskList {
        items: TaskStore::new(pool).list(&query).await?,
    }))
}
