use crate::app::handlers::{ApiError, AppState};
use crate::domains::organizations::enrichment::OrgEnrichmentStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn apply_enrichment(
    State(s): State<AppState>,
    Path((_oid, rid)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    OrgEnrichmentStore::new(pool).apply(&rid).await?;
    Ok(Json(serde_json::json!({"applied":true})))
}
