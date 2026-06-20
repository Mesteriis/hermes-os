use super::*;

// ── Calendar Search ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarSearchQuery {
    q: String,
}

pub(crate) async fn get_calendar_search(
    State(state): State<AppState>,
    Query(query): Query<CalendarSearchQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let results = CalendarBrainService::search_events(&pool, &query.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(results))
}
