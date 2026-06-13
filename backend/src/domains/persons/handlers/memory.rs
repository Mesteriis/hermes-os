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
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let fact = PersonFactStore::new(pool)
        .upsert(
            &person_id,
            &req.fact_type,
            &req.value,
            req.source.as_deref().unwrap_or("manual"),
            req.confidence.unwrap_or(1.0),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(fact))
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
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let card = PersonMemoryCardStore::new(pool)
        .upsert(
            &person_id,
            &req.title,
            &req.description,
            req.source.as_deref().unwrap_or("manual"),
            req.importance.unwrap_or(5),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(card))
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
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pref = PersonPreferenceStore::new(pool)
        .upsert(
            &person_id,
            &req.preference_type,
            &req.value,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(pref))
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
    let event = RelationshipEventStore::new(pool)
        .add(&NewRelationshipEvent { person_id, ..req })
        .await
        .map_err(ApiError::from)?;
    Ok(Json(event))
}
