use super::*;

// ── Calendar Watchtower ────────────────────────────────────────────────────

pub(crate) async fn get_calendar_watchtower(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let preparation = CalendarWatchtowerService::events_needing_preparation(&pool)
        .await
        .map_err(ApiError::from)?;
    let no_outcomes = CalendarWatchtowerService::events_without_outcomes(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        json!({"preparation": preparation, "without_outcomes": no_outcomes}),
    ))
}

pub(crate) async fn get_calendar_health(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let load = CalendarWatchtowerService::meeting_load_analysis(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(load))
}
