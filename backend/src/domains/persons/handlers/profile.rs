use super::support::*;

#[derive(Serialize)]
pub(crate) struct PersonListResponse {
    items: Vec<crate::domains::persons::enrichment::EnrichedPerson>,
}

#[derive(Serialize)]
pub(crate) struct PersonaListResponse {
    items: Vec<PersonaReadModel>,
}

#[derive(Serialize)]
pub(crate) struct PersonaReadModel {
    persona_id: String,
    persona_type: crate::domains::persons::api::PersonaType,
    is_self: bool,
    identity: PersonaIdentityReadModel,
    communication: PersonaCommunicationReadModel,
    compatibility: PersonaCompatibilityReadModel,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub(crate) struct PersonaIdentityReadModel {
    display_name: String,
    email_address: String,
}

#[derive(Serialize)]
pub(crate) struct PersonaCommunicationReadModel {
    primary_email: String,
}

#[derive(Serialize)]
pub(crate) struct PersonaCompatibilityReadModel {
    legacy_person_id: String,
    legacy_route: &'static str,
}

#[derive(Deserialize)]
pub(crate) struct PersonListQuery {
    favorites_only: Option<bool>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct PersonaListQuery {
    limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct PersonaUpdateRequest {
    identity: Option<PersonaIdentityUpdateRequest>,
    is_self: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct PersonaIdentityUpdateRequest {
    display_name: Option<String>,
}

pub(crate) async fn get_persons(
    State(state): State<AppState>,
    Query(query): Query<PersonListQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    let items = store
        .list_enriched(
            query.favorites_only.unwrap_or(false),
            query.limit.unwrap_or(50),
        )
        .await?;
    Ok(Json(PersonListResponse { items }))
}

pub(crate) async fn get_personas(
    State(state): State<AppState>,
    Query(query): Query<PersonaListQuery>,
) -> Result<Json<PersonaListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonProjectionStore::new(pool);
    let items = store
        .list_personas(query.limit.unwrap_or(50))
        .await?
        .into_iter()
        .map(persona_read_model)
        .collect();
    Ok(Json(PersonaListResponse { items }))
}

pub(crate) async fn get_persona(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaReadModel>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonProjectionStore::new(pool);
    match store.get_persona(&persona_id).await? {
        Some(persona) => Ok(Json(persona_read_model(persona))),
        None => Err(ApiError::PersonIdentityNotFound),
    }
}

pub(crate) async fn put_persona(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<PersonaUpdateRequest>,
) -> Result<Json<PersonaReadModel>, ApiError> {
    if req.is_self == Some(false) {
        return Err(ApiError::InvalidPersonaQuery(
            "is_self=false is not supported; set another Persona as owner instead",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let display_name = req
        .identity
        .as_ref()
        .and_then(|identity| identity.display_name.as_deref());
    let persona = PersonProjectionStore::new(pool)
        .update_persona(&persona_id, display_name, req.is_self == Some(true))
        .await?;
    Ok(Json(persona_read_model(persona)))
}

fn persona_read_model(person: Person) -> PersonaReadModel {
    PersonaReadModel {
        persona_id: person.person_id.clone(),
        persona_type: person.persona_type,
        is_self: person.is_self,
        identity: PersonaIdentityReadModel {
            display_name: person.display_name,
            email_address: person.email_address.clone(),
        },
        communication: PersonaCommunicationReadModel {
            primary_email: person.email_address,
        },
        compatibility: PersonaCompatibilityReadModel {
            legacy_person_id: person.person_id,
            legacy_route: "/api/v1/persons",
        },
        created_at: person.created_at,
        updated_at: person.updated_at,
    }
}

pub(crate) async fn get_person(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<crate::domains::persons::enrichment::EnrichedPerson>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    match store.get_enriched(&person_id).await? {
        Some(person) => Ok(Json(person)),
        None => Err(ApiError::PersonIdentityNotFound),
    }
}

#[derive(Serialize)]
pub(crate) struct OwnerPersonaResponse {
    owner_persona: Option<crate::domains::persons::api::Person>,
}

#[derive(Deserialize)]
pub(crate) struct SetOwnerPersonaRequest {
    person_id: String,
}

pub(crate) async fn get_owner_persona(
    State(state): State<AppState>,
) -> Result<Json<OwnerPersonaResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let owner_persona = PersonProjectionStore::new(pool).owner_persona().await?;
    Ok(Json(OwnerPersonaResponse { owner_persona }))
}

pub(crate) async fn put_owner_persona(
    State(state): State<AppState>,
    Json(req): Json<SetOwnerPersonaRequest>,
) -> Result<Json<OwnerPersonaResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let owner_persona = PersonProjectionStore::new(pool)
        .set_owner_persona(&req.person_id)
        .await?;
    Ok(Json(OwnerPersonaResponse {
        owner_persona: Some(owner_persona),
    }))
}

pub(crate) async fn post_person_fingerprint(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let msg_store = crate::domains::mail::messages::MessageProjectionStore::new(pool.clone());
    // Build person messages from this person's email history
    let messages = msg_store.recent_messages(50).await?;
    let person_msgs: Vec<crate::domains::persons::intelligence::PersonMessage> = messages
        .into_iter()
        .filter(|m| {
            m.message.sender.contains(&person_id)
                || m.message.recipients.iter().any(|r| r.contains(&person_id))
        })
        .map(|m| crate::domains::persons::intelligence::PersonMessage {
            subject: m.message.subject,
            body_text: m.message.body_text,
            occurred_at: m.message.occurred_at,
        })
        .collect();
    let fp =
        crate::domains::persons::intelligence::PersonIntelligenceService::heuristic_fingerprint(
            &person_msgs,
        );
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    store.enrich_person(&person_id, &fp).await?;
    Ok(Json(
        serde_json::json!({"enriched": true, "fingerprint": fp}),
    ))
}

pub(crate) async fn post_person_favorite(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    let fav = store.toggle_favorite(&person_id).await?;
    Ok(Json(serde_json::json!({"is_favorite": fav})))
}

#[derive(Deserialize)]
pub(crate) struct PersonNotesRequest {
    notes: String,
}
pub(crate) async fn put_person_notes(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<PersonNotesRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    store.set_notes(&person_id, &req.notes).await?;
    Ok(Json(serde_json::json!({"saved": true})))
}

#[derive(Deserialize)]
pub(crate) struct PersonSearchQuery {
    q: String,
    limit: Option<i64>,
}
pub(crate) async fn get_person_search(
    State(state): State<AppState>,
    Query(query): Query<PersonSearchQuery>,
) -> Result<Json<PersonListResponse>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::InvalidCommunicationQuery("search query required"));
    }
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::persons::enrichment::PersonEnrichmentStore::new(pool);
    let items = store
        .search_persons(&query.q, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(PersonListResponse { items }))
}
