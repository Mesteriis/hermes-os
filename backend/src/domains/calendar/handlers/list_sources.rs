use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::CalendarSourceStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn list_sources(
    State(s): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarSourceStore::new(pool)
        .list_by_account(&account_id)
        .await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
