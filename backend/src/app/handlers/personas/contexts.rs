use super::support::*;
// ── Persona Roles ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaRolesResponse {
    items: Vec<PersonaRole>,
}

pub(crate) async fn get_persona_roles(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaRolesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<PersonaRoleStore>(pool);
    let items = store
        .list_by_person(&persona_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonaRolesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaRoleRequest {
    role: String,
}

pub(crate) async fn post_persona_role(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<NewPersonaRoleRequest>,
) -> Result<Json<PersonaRole>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::command_service::PersonaCommandService::new(pool)
            .assign_role_manual(&persona_id, &req.role)
            .await?,
    ))
}

pub(crate) async fn delete_persona_role(
    State(state): State<AppState>,
    Path((persona_id, role)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = crate::domains::personas::command_service::PersonaCommandService::new(pool)
        .remove_role_manual(&persona_id, &role)
        .await?;
    Ok(Json(json!({"deleted": deleted})))
}

// ── Persona Interaction Contexts ────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaInteractionContextsResponse {
    items: Vec<PersonaInteractionContext>,
}

pub(crate) async fn get_persona_interaction_contexts(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaInteractionContextsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        PersonaInteractionContextStore,
    >(pool);
    let items = store
        .list_by_person(&persona_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonaInteractionContextsResponse { items }))
}

pub(crate) async fn post_persona_interaction_context(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Json(req): Json<NewPersonaInteractionContextRequest>,
) -> Result<Json<PersonaInteractionContext>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        crate::domains::personas::command_service::PersonaCommandService::new(pool)
            .upsert_persona_interaction_context_manual(&NewPersonaInteractionContext {
                source_persona_id: persona_id,
                interaction_context_id: req.interaction_context_id,
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
pub(crate) struct NewPersonaInteractionContextRequest {
    #[serde(alias = "persona_id")]
    interaction_context_id: String,
    name: String,
    context: Option<String>,
    default_tone: Option<String>,
    default_language: Option<String>,
    preferred_channel: Option<String>,
}

pub(crate) async fn delete_persona_interaction_context(
    State(state): State<AppState>,
    Path((source_persona_id, target_persona_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = crate::domains::personas::command_service::PersonaCommandService::new(pool)
        .delete_persona_interaction_context_manual(&source_persona_id, &target_persona_id)
        .await?;
    Ok(Json(json!({"deleted": deleted})))
}
