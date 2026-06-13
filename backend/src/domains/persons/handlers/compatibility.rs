use super::support::*;

// ── Person Roles ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonRolesResponse {
    items: Vec<PersonRole>,
}

pub(crate) async fn get_person_roles(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonRolesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonRoleStore::new(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonRolesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonRoleRequest {
    role: String,
}

pub(crate) async fn post_person_role(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonRoleRequest>,
) -> Result<Json<PersonRole>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonRoleStore::new(pool);
    let role = store
        .assign(&person_id, &req.role, None)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(role))
}

pub(crate) async fn delete_person_role(
    State(state): State<AppState>,
    Path((person_id, role)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonRoleStore::new(pool);
    let deleted = store
        .remove(&person_id, &role)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": deleted})))
}

// ── Person Personas ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonPersonasResponse {
    items: Vec<PersonPersona>,
}

pub(crate) async fn get_person_personas(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
) -> Result<Json<PersonPersonasResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonPersonaStore::new(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPersonasResponse { items }))
}

pub(crate) async fn post_person_persona(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonPersona>,
) -> Result<Json<PersonPersona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonPersonaStore::new(pool);
    let persona = store
        .upsert(&NewPersonPersona { person_id, ..req })
        .await
        .map_err(ApiError::from)?;
    Ok(Json(persona))
}

pub(crate) async fn delete_person_persona(
    State(state): State<AppState>,
    Path((_person_id, persona_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = PersonPersonaStore::new(pool);
    let deleted = store.delete(&persona_id).await.map_err(ApiError::from)?;
    Ok(Json(json!({"deleted": deleted})))
}
