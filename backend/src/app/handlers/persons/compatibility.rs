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
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .assign_role_manual(&person_id, &req.role)
            .await?,
    ))
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
    let deleted = crate::domains::persons::service::PersonCommandService::new(pool)
        .remove_role_manual(&person_id, &role)
        .await?;
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
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_persona_manual(&NewPersonPersona { person_id, ..req })
            .await?,
    ))
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
    let deleted = crate::domains::persons::service::PersonCommandService::new(pool)
        .delete_person_persona_manual(&_person_id, &persona_id)
        .await?;
    Ok(Json(json!({"deleted": deleted})))
}
