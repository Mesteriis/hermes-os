use super::dto::NotesReq;
use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::enrichment::PersonEnrichmentStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn notes(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(r): Json<NotesReq>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    PersonEnrichmentStore::new(pool)
        .set_notes(&id, &r.notes)
        .await?;
    Ok(Json(serde_json::json!({"saved":true})))
}
