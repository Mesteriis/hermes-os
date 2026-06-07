use super::dto::LinkContact;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::core::OrgContactLinkStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn link_contact(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
    Json(r): Json<LinkContact>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = OrgContactLinkStore::new(pool)
        .link(&org_id, &r.person_id, r.role.as_deref(), None)
        .await?;
    Ok(Json(serde_json::to_value(item).unwrap_or_default()))
}
