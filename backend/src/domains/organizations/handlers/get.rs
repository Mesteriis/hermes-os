use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::api::{Organization, OrganizationStore};
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Organization>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    OrganizationStore::new(pool)
        .get(&id)
        .await?
        .ok_or(ApiError::NotFound)
        .map(Json)
}
