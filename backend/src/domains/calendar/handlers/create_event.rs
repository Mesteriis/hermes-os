use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use axum::Json;
use axum::extract::State;
pub(crate) async fn create_event(
    State(s): State<AppState>,
    Json(r): Json<NewCalendarEvent>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool).create(&r).await?;
    Ok(Json(serde_json::to_value(event).unwrap_or_default()))
}
