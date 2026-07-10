use super::support::*;

#[derive(Serialize)]
pub(crate) struct PersonaIdentitiesResponse {
    items: Vec<PersonaIdentityApiResponse>,
}

#[derive(Serialize)]
pub(crate) struct IdentityTracesResponse {
    items: Vec<PersonaIdentityApiResponse>,
}

#[derive(Serialize)]
pub(crate) struct PersonaIdentityApiResponse {
    id: String,
    persona_id: Option<String>,
    identity_type: String,
    identity_value: String,
    source: String,
    confidence: f64,
    last_verified_at: Option<DateTime<Utc>>,
    status: String,
    metadata: Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PersonaIdentity> for PersonaIdentityApiResponse {
    fn from(identity: PersonaIdentity) -> Self {
        Self {
            id: identity.id,
            persona_id: identity.persona_id,
            identity_type: identity.identity_type,
            identity_value: identity.identity_value,
            source: identity.source,
            confidence: identity.confidence,
            last_verified_at: identity.last_verified_at,
            status: identity.status,
            metadata: identity.metadata,
            created_at: identity.created_at,
            updated_at: identity.updated_at,
        }
    }
}

pub(crate) async fn get_persona_identities(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaIdentitiesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonaIdentityStore>(pool);
    let items = store
        .list_by_person(&persona_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonaIdentitiesResponse {
        items: items.into_iter().map(Into::into).collect(),
    }))
}

#[derive(Deserialize)]
pub(crate) struct IdentityTracesQuery {
    status: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_identity_traces(
    State(state): State<AppState>,
    Query(query): Query<IdentityTracesQuery>,
) -> Result<Json<IdentityTracesResponse>, ApiError> {
    if query.status.as_deref().unwrap_or("unattached") != "unattached" {
        return Err(ApiError::InvalidCommunicationQuery(
            "identity trace status must be unattached",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonaIdentityStore>(pool);
    let items = store
        .list_unattached(query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(IdentityTracesResponse {
        items: items.into_iter().map(Into::into).collect(),
    }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaIdentityRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct NewIdentityTraceRequest {
    identity_type: String,
    identity_value: String,
    source: Option<String>,
}

pub(crate) async fn post_identity_trace(
    State(state): State<AppState>,
    Json(req): Json<NewIdentityTraceRequest>,
) -> Result<Json<PersonaIdentityApiResponse>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::service::PersonaCommandService::new(pool)
            .create_identity_trace_manual(&req.identity_type, &req.identity_value, requested_source)
            .await?
            .into(),
    ))
}

#[derive(Deserialize)]
pub(crate) struct IdentityTraceAssignmentRequest {
    #[serde(alias = "person_id")]
    persona_id: String,
}

pub(crate) async fn put_identity_trace_assignment(
    State(state): State<AppState>,
    Path(identity_id): Path<String>,
    Json(req): Json<IdentityTraceAssignmentRequest>,
) -> Result<Json<PersonaIdentityApiResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::service::PersonaCommandService::new(pool)
            .assign_identity_trace_manual(&identity_id, &req.persona_id)
            .await?
            .into(),
    ))
}

pub(crate) async fn post_persona_identity(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<NewPersonaIdentityRequest>,
) -> Result<Json<PersonaIdentityApiResponse>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::service::PersonaCommandService::new(pool)
            .upsert_persona_identity_manual(
                &persona_id,
                &req.identity_type,
                &req.identity_value,
                requested_source,
            )
            .await?
            .into(),
    ))
}

pub(crate) async fn delete_persona_identity(
    State(state): State<AppState>,
    Path((persona_id, identity_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = crate::domains::personas::service::PersonaCommandService::new(pool)
        .delete_persona_identity_manual(&persona_id, &identity_id)
        .await?;
    Ok(Json(json!({"deleted": deleted})))
}
