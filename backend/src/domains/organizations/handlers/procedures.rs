use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::workflows::OrgProcedureStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn procedures(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(OrgProcedureStore::new(pool).list(&org_id).await?).unwrap_or_default(),
    ))
}
