use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::core::OrgDepartmentStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn list_departments(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = OrgDepartmentStore::new(pool).list(&org_id).await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
