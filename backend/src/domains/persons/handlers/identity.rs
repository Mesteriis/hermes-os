use super::support::*;

#[derive(Serialize)]
pub(crate) struct PersonIdentitiesResponse {
    items: Vec<PersonIdentity>,
}

#[derive(Serialize)]
pub(crate) struct IdentityTracesResponse {
    items: Vec<PersonIdentity>,
}

pub(crate) async fn get_person_identities(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonIdentitiesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonsIdentityStore::new(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonIdentitiesResponse { items }))
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
    let store = PersonsIdentityStore::new(pool);
    let items = store
        .list_unattached(query.limit.unwrap_or(50))
        .await
        .map_err(ApiError::from)?;
    Ok(Json(IdentityTracesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonIdentityRequest {
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
) -> Result<Json<PersonIdentity>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonsIdentityStore::new(pool);
    let identity = store
        .create_unattached(
            &req.identity_type,
            &req.identity_value,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(identity))
}

#[derive(Deserialize)]
pub(crate) struct IdentityTraceAssignmentRequest {
    person_id: String,
}

pub(crate) async fn put_identity_trace_assignment(
    State(state): State<AppState>,
    Path(identity_id): Path<String>,
    Json(req): Json<IdentityTraceAssignmentRequest>,
) -> Result<Json<PersonIdentity>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonsIdentityStore::new(pool);
    let identity = store
        .attach_to_persona(&identity_id, &req.person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(identity))
}

pub(crate) async fn post_person_identity(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonIdentityRequest>,
) -> Result<Json<PersonIdentity>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonsIdentityStore::new(pool);
    let identity = store
        .upsert(
            &person_id,
            &req.identity_type,
            &req.identity_value,
            req.source.as_deref().unwrap_or("manual"),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(identity))
}

pub(crate) async fn delete_person_identity(
    State(state): State<AppState>,
    Path((_person_id, identity_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonsIdentityStore::new(pool);
    let deleted = store.delete(&identity_id).await.map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": deleted})))
}
