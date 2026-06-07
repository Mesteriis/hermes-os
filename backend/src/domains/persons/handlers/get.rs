use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::enrichment::PersonEnrichmentStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(PersonEnrichmentStore::new(pool).get_enriched(&id).await?)
            .unwrap_or_default(),
    ))
}
