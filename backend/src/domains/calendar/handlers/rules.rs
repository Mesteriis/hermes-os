use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::rules::CalendarRuleStore;
use axum::Json;
use axum::extract::State;
pub async fn rules(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarRuleStore::new(pool).list().await?;
    Ok(Json(serde_json::to_value(items).unwrap_or_default()))
}
