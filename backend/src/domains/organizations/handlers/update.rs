use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::api::{Organization, OrganizationStore, OrganizationUpdate};
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(r): Json<OrganizationUpdate>,
) -> Result<Json<Organization>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(OrganizationStore::new(pool).update(&id, &r).await?))
}
