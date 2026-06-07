use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::health::OrgHealthStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn health(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(OrgHealthStore::new(pool).get(&org_id).await?).unwrap_or_default(),
    ))
}
