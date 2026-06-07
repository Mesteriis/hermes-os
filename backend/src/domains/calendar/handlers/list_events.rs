use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::{CalendarEventListQuery, CalendarEventStore};
use axum::Json;
use axum::extract::{Query, State};
pub(crate) async fn list_events(
    State(s): State<AppState>,
    Query(q): Query<CalendarEventListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarEventStore::new(pool).list(&q).await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
