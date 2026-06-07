use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::core::OrgIdentityStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn list_identities(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = OrgIdentityStore::new(pool).list(&org_id).await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
