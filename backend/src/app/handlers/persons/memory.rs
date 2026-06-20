use super::support::*;

// ── Person Facts ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonFactsResponse {
    items: Vec<crate::domains::persons::memory::PersonFact>,
}

pub(crate) async fn get_person_facts(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonFactsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonFactStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonFactsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonFactRequest {
    fact_type: String,
    value: String,
    source: Option<String>,
    confidence: Option<f64>,
}

pub(crate) async fn post_person_fact(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonFactRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonFact>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_fact_manual(
                &person_id,
                &req.fact_type,
                &req.value,
                requested_source,
                req.confidence.unwrap_or(1.0),
            )
            .await?,
    ))
}

// ── Person Memory Cards ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonMemoryCardsResponse {
    items: Vec<crate::domains::persons::memory::PersonMemoryCard>,
}

pub(crate) async fn get_person_memory_cards(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonMemoryCardsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonMemoryCardStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonMemoryCardsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonMemoryCardRequest {
    title: String,
    description: String,
    source: Option<String>,
    importance: Option<i16>,
}

pub(crate) async fn post_person_memory_card(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonMemoryCardRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonMemoryCard>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_memory_card_manual(
                &person_id,
                &req.title,
                &req.description,
                requested_source,
                req.importance.unwrap_or(5),
            )
            .await?,
    ))
}

// ── Person Preferences ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPreferencesResponse {
    items: Vec<crate::domains::persons::memory::PersonPreference>,
}

pub(crate) async fn get_person_preferences(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPreferencesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = PersonPreferenceStore::new(pool)
        .list(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPreferencesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonPreferenceRequest {
    preference_type: String,
    value: String,
    source: Option<String>,
}

pub(crate) async fn post_person_preference(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonPreferenceRequest>,
) -> Result<Json<crate::domains::persons::memory::PersonPreference>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_preference_manual(
                &person_id,
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
    items: Vec<crate::domains::persons::memory::RelationshipEvent>,
}

pub(crate) async fn get_person_timeline(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<RelationshipTimelineResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = RelationshipEventStore::new(pool)
        .timeline(&person_id, query.limit.unwrap_or(50))
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
    Path(person_id): Path<String>,
    Json(req): Json<NewRelationshipEvent>,
) -> Result<Json<crate::domains::persons::memory::RelationshipEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .add_relationship_event_manual(&NewRelationshipEvent { person_id, ..req })
            .await?,
    ))
}
