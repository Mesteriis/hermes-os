use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::{CalendarEventStore, CalendarEventUpdate};
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn update_event(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(r): Json<CalendarEventUpdate>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool).update(&id, &r).await?;
    Ok(Json(serde_json::to_value(event).unwrap_or_default()))
}
