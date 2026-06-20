use super::*;

// ── Analytics: Time Distribution ───────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct AnalyticsRangeQuery {
    from: Option<String>,
    to: Option<String>,
}

pub(crate) async fn get_time_distribution(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let dist = CalendarWatchtowerService::time_distribution(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dist))
}

pub(crate) async fn get_focus_balance(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let balance = CalendarWatchtowerService::focus_balance(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(balance))
}

pub(crate) async fn get_back_to_back(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let b2b = CalendarWatchtowerService::back_to_back_meetings(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(b2b))
}
