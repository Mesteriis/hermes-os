use crate::app::handlers::{ApiError, AppState};
use crate::domains::tasks::rules::TaskRuleStore;
use axum::Json;
use axum::extract::State;
pub(crate) async fn rules(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(TaskRuleStore::new(pool).list().await?).unwrap_or_default(),
    ))
}
