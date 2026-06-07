use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::events::CalendarAccountStore;
use axum::Json;
use axum::extract::{Path, State};
pub(crate) async fn get_account(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let account = CalendarAccountStore::new(pool)
        .get(&id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(serde_json::to_value(account).unwrap_or_default()))
}
