use super::*;

// ── Calendar Brain ─────────────────────────────────────────────────────────

pub(crate) async fn get_event_brief(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = CalendarBrainService::meeting_brief(&pool, &event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(brief))
}

pub(crate) async fn post_generate_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = CalendarBrainService::generate_agenda(&pool, &event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(agenda))
}

// ── Weekly Brief ───────────────────────────────────────────────────────────

pub(crate) async fn get_weekly_brief(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = CalendarWatchtowerService::weekly_brief(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(brief))
}

// ── Calendar Analytics ─────────────────────────────────────────────────────

pub(crate) async fn get_calendar_analytics(
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

// ── Calendar Brain ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarBrainQueryParams {
    q: String,
}

pub(crate) async fn post_calendar_brain(
    State(state): State<AppState>,
    Json(req): Json<CalendarBrainQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let answer = CalendarBrainService::answer(&pool, &req.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(answer))
}
