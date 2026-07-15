use super::*;

// ── Deadlines ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct DeadlinesResponse {
    items: Vec<crate::domains::calendar::scheduling::DeadlineEvent>,
}

#[derive(Deserialize)]
pub(crate) struct DeadlineQuery {
    status: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_deadlines(
    State(state): State<AppState>,
    Query(query): Query<DeadlineQuery>,
) -> Result<Json<DeadlinesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::stores::domain_stores::app_store::<DeadlineStore>(pool)
        .list(query.status.as_deref(), query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(DeadlinesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewDeadlineRequest {
    title: String,
    due_at: DateTime<Utc>,
    severity: Option<String>,
    source_entity_type: Option<String>,
    source_entity_id: Option<String>,
}

pub(crate) async fn post_deadline(
    State(state): State<AppState>,
    Json(req): Json<NewDeadlineRequest>,
) -> Result<Json<crate::domains::calendar::scheduling::DeadlineEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let d = CalendarCommandService::new(pool)
        .create_deadline_manual(
            &req.title,
            req.due_at,
            req.severity.as_deref(),
            req.source_entity_type.as_deref(),
            req.source_entity_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(d))
}

// ── Focus Blocks ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct FocusBlocksResponse {
    items: Vec<crate::domains::calendar::scheduling::FocusBlock>,
}

#[derive(Deserialize)]
pub(crate) struct FocusBlockQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: Option<i64>,
}

pub(crate) async fn get_focus_blocks(
    State(state): State<AppState>,
    Query(query): Query<FocusBlockQuery>,
) -> Result<Json<FocusBlocksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::stores::domain_stores::app_store::<FocusBlockStore>(pool)
        .list(query.from, query.to, query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(FocusBlocksResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewFocusBlockRequest {
    title: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    purpose: Option<String>,
    linked_project_id: Option<String>,
    protection_level: Option<String>,
}

pub(crate) async fn post_focus_block(
    State(state): State<AppState>,
    Json(req): Json<NewFocusBlockRequest>,
) -> Result<Json<crate::domains::calendar::scheduling::FocusBlock>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let fb = CalendarCommandService::new(pool)
        .create_focus_block_manual(
            &req.title,
            req.start_at,
            req.end_at,
            req.purpose.as_deref(),
            req.linked_project_id.as_deref(),
            req.protection_level.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fb))
}

// ── Smart Schedule ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct SmartScheduleRequest {
    duration_minutes: Option<i64>,
    lookahead_hours: Option<i64>,
}

pub(crate) async fn post_smart_schedule(
    State(state): State<AppState>,
    Json(req): Json<SmartScheduleRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let events =
        crate::app::api_support::stores::domain_stores::app_store::<CalendarEventStore>(pool)
            .list(&CalendarEventListQuery {
                limit: Some(200),
                ..Default::default()
            })
            .await?;
    let pairs: Vec<(DateTime<Utc>, DateTime<Utc>)> =
        events.iter().map(|e| (e.start_at, e.end_at)).collect();
    let slots = SmartSchedulingService::find_slots(
        &pairs,
        req.duration_minutes.unwrap_or(30),
        req.lookahead_hours.unwrap_or(48),
    );
    Ok(Json(json!({"slots": slots})))
}
