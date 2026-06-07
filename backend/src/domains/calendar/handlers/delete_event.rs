use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::CalendarEventStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn delete_event(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool).delete(&id).await?;
    Ok(Json(serde_json::json!({"deleted":true})))
}
