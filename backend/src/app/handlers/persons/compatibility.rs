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
    let store = crate::app::api_support::app_store::<PersonRoleStore>(pool);
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
    let store = crate::app::api_support::app_store::<PersonPersonaStore>(pool);
    let items = store
        .list_by_person(&person_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonPersonasResponse { items }))
}

pub(crate) async fn post_person_persona(
    State(state): State<AppState>,
    Path(person_id): Path<String>,
    Json(req): Json<NewPersonPersonaRequest>,
) -> Result<Json<PersonPersona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::persons::service::PersonCommandService::new(pool)
            .upsert_person_persona_manual(&NewPersonPersona {
                person_id,
                persona_id: req.persona_id,
                name: req.name,
                context: req.context,
                default_tone: req.default_tone,
                default_language: req.default_language,
                preferred_channel: req.preferred_channel,
            })
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonPersonaRequest {
    persona_id: String,
    name: String,
    context: Option<String>,
    default_tone: Option<String>,
    default_language: Option<String>,
    preferred_channel: Option<String>,
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
