use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::core::EventAgendaStore;
use axum::Json;
use axum::extract::{Path, State};
pub async fn agenda(
    State(s): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = EventAgendaStore::new(pool).get(&event_id).await?;
    Ok(Json(serde_json::to_value(agenda).unwrap_or_default()))
}
