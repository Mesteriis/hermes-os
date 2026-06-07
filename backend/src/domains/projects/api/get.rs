use crate::app::handlers::{ApiError, AppState};
use crate::domains::projects::core::{ProjectDetail, ProjectStore};
use axum::Json;
use axum::extract::{Path, State};

pub(crate) async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProjectDetail>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    ProjectStore::new(pool)
        .project_detail(&id)
        .await?
        .ok_or(ApiError::ProjectNotFound)
        .map(Json)
}
