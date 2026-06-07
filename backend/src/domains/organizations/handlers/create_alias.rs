use super::dto::NewAlias;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::core::OrgAliasStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn create_alias(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
    Json(r): Json<NewAlias>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let item = OrgAliasStore::new(pool)
        .add(&org_id, &r.alias, "manual", "manual")
        .await?;
    Ok(Json(serde_json::to_value(item).unwrap_or_default()))
}
