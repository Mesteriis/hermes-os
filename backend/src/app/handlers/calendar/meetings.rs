use super::*;

// ── Meeting Notes ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct MeetingNotesResponse {
    items: Vec<crate::domains::calendar::meetings::MeetingNote>,
}

pub(crate) async fn get_meeting_notes(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<MeetingNotesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = MeetingNoteStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(MeetingNotesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewNoteRequest {
    content: String,
    format: Option<String>,
    source: Option<String>,
}

pub(crate) async fn post_meeting_note(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewNoteRequest>,
) -> Result<Json<crate::domains::calendar::meetings::MeetingNote>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let note = CalendarCommandService::new(pool)
        .create_meeting_note_manual(
            &event_id,
            &req.content,
            req.format.as_deref(),
            requested_source,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(note))
}

// ── Meeting Outcomes ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct MeetingOutcomesResponse {
    items: Vec<crate::domains::calendar::meetings::MeetingOutcome>,
}

pub(crate) async fn get_meeting_outcomes(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<MeetingOutcomesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = MeetingOutcomeStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(MeetingOutcomesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewOutcomeRequest {
    outcome_type: String,
    title: String,
    description: Option<String>,
    owner_person_id: Option<String>,
    due_date: Option<DateTime<Utc>>,
}

pub(crate) async fn post_meeting_outcome(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewOutcomeRequest>,
) -> Result<Json<crate::domains::calendar::meetings::MeetingOutcome>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let outcome = CalendarCommandService::new(pool)
        .add_meeting_outcome_manual(
            &event_id,
            &req.outcome_type,
            &req.title,
            req.description.as_deref(),
            req.owner_person_id.as_deref(),
            req.due_date,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(outcome))
}

pub(crate) async fn post_event_follow_up(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarEventStore::new(pool.clone())
        .set_status_manual(
            &event_id,
            "needs_follow_up",
            "calendar_api.post_event_follow_up",
        )
        .await?;
    Ok(Json(json!({"follow_up_created": true})))
}

pub(crate) async fn get_event_follow_up_status(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let status = MeetingOutcomeStore::new(pool)
        .follow_up_status(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(status))
}

// ── Event Recordings ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EventRecordingsResponse {
    items: Vec<crate::domains::calendar::meetings::EventRecording>,
}

pub(crate) async fn get_event_recordings(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRecordingsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = EventRecordingStore::new(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRecordingsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRecordingRequest {
    file_path: Option<String>,
    source: Option<String>,
    duration_seconds: Option<i32>,
}

pub(crate) async fn post_event_recording(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewRecordingRequest>,
) -> Result<Json<crate::domains::calendar::meetings::EventRecording>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let rec = CalendarCommandService::new(pool)
        .add_event_recording_manual(
            &event_id,
            req.file_path.as_deref(),
            requested_source,
            req.duration_seconds,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rec))
}

pub(crate) async fn get_event_transcript(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let t = EventTranscriptStore::new(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&t).unwrap_or_default()))
}
