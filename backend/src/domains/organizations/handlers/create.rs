use super::dto::NewOrg;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::api::{Organization, OrganizationStore};
use axum::Json;
use axum::extract::State;
pub(crate) async fn create(
    State(s): State<AppState>,
    Json(r): Json<NewOrg>,
) -> Result<Json<Organization>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        OrganizationStore::new(pool)
            .create(&r.name, r.kind.as_deref())
            .await?,
    ))
}
