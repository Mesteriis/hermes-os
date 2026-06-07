use super::dto::{ProjectListResponse, ProjectsQuery};
use crate::app::handlers::{ApiError, AppState};
use crate::domains::projects::core::ProjectStore;
use axum::Json;
use axum::extract::{Query, State};

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ProjectsQuery>,
) -> Result<Json<ProjectListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = ProjectStore::new(pool).list_projects(q.limit).await?;
    Ok(Json(ProjectListResponse { items }))
}
