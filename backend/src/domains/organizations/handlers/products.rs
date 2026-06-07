use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::finance::OrgProductStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn products(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(OrgProductStore::new(pool).list(&org_id).await?).unwrap_or_default(),
    ))
}
