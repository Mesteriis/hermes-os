use super::*;

// ── Calendar Events ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct CalendarEventsResponse {
    items: Vec<crate::domains::calendar::events::CalendarEvent>,
}

#[derive(Deserialize)]
pub(crate) struct CalendarEventQuery {
    account_id: Option<String>,
    source_id: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    status: Option<String>,
    event_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_calendar_events(
    State(state): State<AppState>,
    Query(query): Query<CalendarEventQuery>,
) -> Result<Json<CalendarEventsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let list_query = CalendarEventListQuery {
        account_id: query.account_id,
        source_id: query.source_id,
        from: query.from,
        to: query.to,
        status: query.status,
        event_type: query.event_type,
        limit: query.limit,
    };
    let items = CalendarEventStore::new(pool).list(&list_query).await?;
    Ok(Json(CalendarEventsResponse { items }))
}

pub(crate) async fn post_calendar_event(
    State(state): State<AppState>,
    Json(req): Json<NewCalendarEvent>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool).create(&req).await?;
    Ok(Json(event))
}

pub(crate) async fn get_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool)
        .get(&event_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(update): Json<CalendarEventUpdate>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool)
        .update(&event_id, &update)
        .await?;
    Ok(Json(event))
}

pub(crate) async fn delete_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool).delete(&event_id).await?;
    Ok(Json(json!({"deleted": true})))
}

// ── Calendar Event Reschedule / Cancel ─────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct RescheduleRequest {
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

pub(crate) async fn post_calendar_event_reschedule(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<RescheduleRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool)
        .reschedule(&event_id, req.start_at, req.end_at)
        .await?;
    Ok(Json(event))
}

pub(crate) async fn post_calendar_event_cancel(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool)
        .set_status(&event_id, "cancelled")
        .await?;
    Ok(Json(json!({"cancelled": true})))
}

// ── Event Participants ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventParticipantsResponse {
    items: Vec<crate::domains::calendar::core::EventParticipant>,
}

pub(crate) async fn get_event_participants(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventParticipantsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EventParticipantStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventParticipantsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewParticipantRequest {
    email: String,
    display_name: Option<String>,
    role: Option<String>,
    person_id: Option<String>,
    organization_id: Option<String>,
}

pub(crate) async fn post_event_participant(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewParticipantRequest>,
) -> Result<Json<crate::domains::calendar::core::EventParticipant>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let p = EventParticipantStore::new(pool)
        .add(
            &event_id,
            &req.email,
            req.display_name.as_deref(),
            req.role.as_deref(),
            req.person_id.as_deref(),
            req.organization_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(p))
}

// ── Event Relations ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventRelationsResponse {
    items: Vec<crate::domains::calendar::core::EventRelation>,
}

pub(crate) async fn get_event_relations(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRelationsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EventRelationStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRelationsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRelationRequest {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}

pub(crate) async fn post_event_relation(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewRelationRequest>,
) -> Result<Json<crate::domains::calendar::core::EventRelation>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rel = EventRelationStore::new(pool)
        .link(
            &event_id,
            &req.entity_type,
            &req.entity_id,
            &req.relation_type,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rel))
}

// ── Event Context Pack ─────────────────────────────────────────────────────

pub(crate) async fn get_event_context_pack(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = EventContextPackStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}

pub(crate) async fn post_event_context_pack(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<ContextPackInput>,
) -> Result<Json<crate::domains::calendar::core::EventContextPack>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = EventContextPackStore::new(pool)
        .upsert(&event_id, &req)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(pack))
}

// ── Event Agenda ───────────────────────────────────────────────────────────

pub(crate) async fn get_event_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = EventAgendaStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&agenda).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetAgendaRequest {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_event_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<SetAgendaRequest>,
) -> Result<Json<crate::domains::calendar::core::EventAgenda>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = EventAgendaStore::new(pool)
        .set(
            &event_id,
            req.items,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(agenda))
}

// ── Event Checklist ────────────────────────────────────────────────────────

pub(crate) async fn get_event_checklist(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let cl = EventChecklistStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&cl).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetChecklistRequest {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_event_checklist(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<SetChecklistRequest>,
) -> Result<Json<crate::domains::calendar::core::EventChecklist>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let cl = EventChecklistStore::new(pool)
        .set(
            &event_id,
            req.items,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(cl))
}
