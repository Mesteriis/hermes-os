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
    let store = crate::app::api_support::app_store::<PersonsIdentityStore>(pool);
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
    let store = crate::app::api_support::app_store::<PersonsIdentityStore>(pool);
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
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .create_identity_trace_manual(&req.identity_type, &req.identity_value, requested_source)
            .await?,
    ))
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
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .assign_identity_trace_manual(&identity_id, &req.person_id)
            .await?,
    ))
}

pub(crate) async fn post_person_identity(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonIdentityRequest>,
) -> Result<Json<PersonIdentity>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_identity_manual(
                &person_id,
                &req.identity_type,
                &req.identity_value,
                requested_source,
            )
            .await?,
    ))
}

pub(crate) async fn delete_person_identity(
    State(state): State<AppState>,
    Path((person_id, identity_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = crate::domains::persons::service::PersonCommandService::new(pool)
        .delete_person_identity_manual(&person_id, &identity_id)
        .await?;
    Ok(Json(json!({"deleted": deleted})))
}
