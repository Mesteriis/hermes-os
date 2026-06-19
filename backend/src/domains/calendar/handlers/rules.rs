use super::*;

// ── Calendar Rules ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct CalendarRulesResponse {
    items: Vec<crate::domains::calendar::rules::CalendarRule>,
}

pub(crate) async fn get_calendar_rules(
    State(state): State<AppState>,
) -> Result<Json<CalendarRulesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = CalendarRuleStore::new(pool)
        .list()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(CalendarRulesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRuleRequest {
    name: String,
    description: Option<String>,
    dsl: Value,
    approval_mode: Option<String>,
}

pub(crate) async fn post_calendar_rule(
    State(state): State<AppState>,
    Json(req): Json<NewRuleRequest>,
) -> Result<Json<crate::domains::calendar::rules::CalendarRule>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rule = CalendarCommandService::new(pool)
        .create_calendar_rule_manual(
            &req.name,
            req.description.as_deref(),
            req.dsl,
            req.approval_mode.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rule))
}

pub(crate) async fn put_calendar_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(update): Json<RuleUpdate>,
) -> Result<Json<crate::domains::calendar::rules::CalendarRule>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rule = CalendarCommandService::new(pool)
        .update_calendar_rule_manual(&rule_id, &update)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rule))
}

pub(crate) async fn delete_calendar_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarCommandService::new(pool)
        .delete_calendar_rule_manual(&rule_id)
        .await?;
    Ok(Json(json!({"deleted": true})))
}
