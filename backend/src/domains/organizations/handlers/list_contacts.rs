use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::core::OrgContactLinkStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn list_contacts(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = OrgContactLinkStore::new(pool).list_by_org(&org_id).await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
