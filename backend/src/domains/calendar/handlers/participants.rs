use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::core::EventParticipantStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn participants(
    State(s): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EventParticipantStore::new(pool).list(&event_id).await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
