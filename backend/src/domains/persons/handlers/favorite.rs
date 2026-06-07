use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::enrichment::PersonEnrichmentStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn favorite(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = PersonEnrichmentStore::new(pool)
        .toggle_favorite(&id)
        .await?;
    Ok(Json(serde_json::json!({"favorite":result})))
}
