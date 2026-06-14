use super::super::*;

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
    let participant = EventParticipantStore::new(pool)
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
    Ok(Json(participant))
}
