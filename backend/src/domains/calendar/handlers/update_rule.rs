use crate::app::handlers::{ApiError, AppState};
use crate::domains::calendar::rules::{CalendarRuleStore, RuleUpdate};
use axum::Json;
use axum::extract::{Path, State};

pub(crate) async fn update_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(req): Json<RuleUpdate>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rule = CalendarRuleStore::new(pool).update(&rule_id, &req).await?;
    Ok(Json(serde_json::to_value(rule).unwrap_or_default()))
}
