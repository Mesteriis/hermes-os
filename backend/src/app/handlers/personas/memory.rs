use super::support::*;

// ── Persona Facts ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaFactsResponse {
    items: Vec<crate::domains::personas::memory::PersonaFact>,
}

pub(crate) async fn get_persona_facts(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaFactsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::stores::domain_stores::app_store::<PersonaFactStore>(pool)
        .list(&persona_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonaFactsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaFactRequest {
    fact_type: String,
    value: String,
    source: Option<String>,
    confidence: Option<f64>,
}

pub(crate) async fn post_persona_fact(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<NewPersonaFactRequest>,
) -> Result<Json<crate::domains::personas::memory::PersonaFact>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::command_service::PersonaCommandService::new(pool)
            .upsert_persona_fact_manual(
                &persona_id,
                &req.fact_type,
                &req.value,
                requested_source,
                req.confidence.unwrap_or(1.0),
            )
            .await?,
    ))
}

// ── Persona Memory Cards ────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaMemoryCardsResponse {
    items: Vec<crate::domains::personas::memory::PersonaMemoryCard>,
}

pub(crate) async fn get_persona_memory_cards(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaMemoryCardsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<PersonaMemoryCardStore>(pool)
            .list(&persona_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(PersonaMemoryCardsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaMemoryCardRequest {
    title: String,
    description: String,
    source: Option<String>,
    importance: Option<i16>,
}

pub(crate) async fn post_persona_memory_card(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<NewPersonaMemoryCardRequest>,
) -> Result<Json<crate::domains::personas::memory::PersonaMemoryCard>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::command_service::PersonaCommandService::new(pool)
            .upsert_persona_memory_card_manual(
                &persona_id,
                &req.title,
                &req.description,
                requested_source,
                req.importance.unwrap_or(5),
            )
            .await?,
    ))
}

// ── Persona Preferences ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaPreferencesResponse {
    items: Vec<crate::domains::personas::memory::PersonaPreference>,
}

pub(crate) async fn get_persona_preferences(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaPreferencesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<PersonaPreferenceStore>(pool)
            .list(&persona_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(PersonaPreferencesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaPreferenceRequest {
    preference_type: String,
    value: String,
    source: Option<String>,
}

pub(crate) async fn post_persona_preference(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<NewPersonaPreferenceRequest>,
) -> Result<Json<crate::domains::personas::memory::PersonaPreference>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::command_service::PersonaCommandService::new(pool)
            .upsert_persona_preference_manual(
                &persona_id,
                &req.preference_type,
                &req.value,
                requested_source,
            )
            .await?,
    ))
}

// ── Relationship Timeline ───────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct RelationshipTimelineResponse {
    items: Vec<crate::domains::personas::memory::RelationshipEvent>,
}

pub(crate) async fn get_persona_timeline(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<RelationshipTimelineResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<RelationshipEventStore>(pool)
            .timeline(&persona_id, query.limit.unwrap_or(50))
            .await
            .map_err(ApiError::from)?;
    Ok(Json(RelationshipTimelineResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct TimelineQuery {
    limit: Option<i64>,
}

pub(crate) async fn post_relationship_event(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<NewRelationshipEventRequest>,
) -> Result<Json<crate::domains::personas::memory::RelationshipEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::command_service::PersonaCommandService::new(pool)
            .add_relationship_event_manual(&NewRelationshipEvent {
                persona_id,
                event_type: req.event_type,
                title: req.title,
                description: req.description,
                occurred_at: req.occurred_at,
                source: req.source,
                related_entity_id: req.related_entity_id,
                related_entity_kind: req.related_entity_kind,
            })
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct NewRelationshipEventRequest {
    event_type: String,
    title: String,
    description: Option<String>,
    occurred_at: DateTime<Utc>,
    source: String,
    related_entity_id: Option<String>,
    related_entity_kind: Option<String>,
}
