use super::*;

// ── Event Intelligence ─────────────────────────────────────────────────────

pub(crate) async fn post_event_classify(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let participants = EventParticipantStore::new(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let event_type = CalendarIntelligenceService::classify_event(
        &event.title,
        participants.len(),
        (event.end_at - event.start_at).num_minutes(),
    );
    let update = CalendarEventUpdate {
        event_type: Some(event_type.clone()),
        ..Default::default()
    };
    CalendarEventStore::new(pool)
        .update(&event_id, &update)
        .await?;
    Ok(Json(json!({"event_type": event_type})))
}

pub(crate) async fn post_event_analyze(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let parts = EventParticipantStore::new(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let has_agenda = EventAgendaStore::new(pool.clone())
        .get(&event_id)
        .await
        .map(|a| a.is_some())
        .unwrap_or(false);
    let has_checklist = EventChecklistStore::new(pool.clone())
        .get(&event_id)
        .await
        .map(|c| c.is_some())
        .unwrap_or(false);
    let has_relations = EventRelationStore::new(pool.clone())
        .list(&event_id)
        .await
        .map(|r| !r.is_empty())
        .unwrap_or(false);

    let importance = CalendarIntelligenceService::calculate_importance(
        &event.title,
        parts.len(),
        has_relations,
        false,
    );
    let readiness = CalendarIntelligenceService::calculate_readiness(
        has_agenda,
        false,
        has_relations,
        has_checklist,
        !parts.is_empty(),
    );
    let risks = CalendarIntelligenceService::detect_risks(
        has_agenda,
        false,
        !parts.is_empty(),
        has_relations,
        event.start_at < Utc::now() + chrono::Duration::hours(24),
    );

    let update = CalendarEventUpdate {
        importance_score: Some(importance),
        readiness_score: Some(readiness),
        ..Default::default()
    };
    CalendarEventStore::new(pool.clone())
        .update(&event_id, &update)
        .await?;

    Ok(Json(
        json!({"importance": importance, "readiness": readiness, "risks": risks}),
    ))
}

pub(crate) async fn get_event_risks(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = CalendarEventStore::new(pool.clone())
        .get(&event_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let parts = EventParticipantStore::new(pool.clone())
        .list(&event_id)
        .await
        .unwrap_or_default();
    let has_agenda = EventAgendaStore::new(pool.clone())
        .get(&event_id)
        .await
        .map(|a| a.is_some())
        .unwrap_or(false);
    let has_relations = EventRelationStore::new(pool.clone())
        .list(&event_id)
        .await
        .map(|r| !r.is_empty())
        .unwrap_or(false);
    let is_soon = event.start_at < Utc::now() + chrono::Duration::hours(24);
    let risks = CalendarIntelligenceService::detect_risks(
        has_agenda,
        false,
        !parts.is_empty(),
        has_relations,
        is_soon,
    );
    Ok(Json(json!({"risks": risks})))
}
