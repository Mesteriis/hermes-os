use super::super::support::*;
use super::models::{PersonaReadModel, persona_read_model};

#[derive(Serialize)]
pub(crate) struct OwnerPersonaResponse {
    owner_persona: Option<PersonaReadModel>,
}

#[derive(Deserialize)]
pub(crate) struct SetOwnerPersonaRequest {
    #[serde(alias = "person_id")]
    persona_id: String,
}

pub(crate) async fn get_owner_persona(
    State(state): State<AppState>,
) -> Result<Json<OwnerPersonaResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let owner_persona = crate::app::api_support::app_store::<PersonaProjectionStore>(pool)
        .owner_persona()
        .await?;
    Ok(Json(OwnerPersonaResponse {
        owner_persona: owner_persona.map(persona_read_model),
    }))
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
    let owner_persona = crate::domains::personas::service::PersonaCommandService::new(pool)
        .set_owner_persona_manual(&req.persona_id)
        .await?;
    Ok(Json(OwnerPersonaResponse {
        owner_persona: Some(persona_read_model(owner_persona)),
    }))
}
