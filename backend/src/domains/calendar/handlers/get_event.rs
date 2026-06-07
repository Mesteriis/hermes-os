use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::CalendarEventStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn get_event(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool)
        .get(&id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(serde_json::to_value(event).unwrap_or_default()))
}
