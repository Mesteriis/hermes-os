use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::{CalendarAccountStore, CalendarAccountUpdate};
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn update_account(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(r): Json<CalendarAccountUpdate>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let account = CalendarAccountStore::new(pool).update(&id, &r).await?;
    Ok(Json(serde_json::to_value(account).unwrap_or_default()))
}
