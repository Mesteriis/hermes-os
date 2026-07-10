use super::super::support::*;
use super::models::{PersonaListResponse, PersonaReadModel, persona_read_model};
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

pub(crate) async fn get_personas(
    State(state): State<AppState>,
    Query(query): Query<PersonaListQuery>,
) -> Result<Json<PersonaListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<PersonaProjectionStore>(pool);
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
    let store = crate::app::api_support::app_store::<PersonaProjectionStore>(pool);
    match store.get_persona(&persona_id).await? {
        Some(persona) => Ok(Json(persona_read_model(persona))),
        None => Err(ApiError::PersonaIdentityNotFound),
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
    let persona = crate::domains::personas::service::PersonaCommandService::new(pool)
        .update_persona_manual(&persona_id, display_name, req.is_self == Some(true))
        .await?;
    Ok(Json(persona_read_model(persona)))
}
