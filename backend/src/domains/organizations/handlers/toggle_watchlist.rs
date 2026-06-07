use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::health::OrgHealthStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn toggle_watchlist(
    State(s): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    OrgHealthStore::new(pool).toggle_watchlist(&org_id).await?;
    Ok(Json(serde_json::json!({"toggled":true})))
}
