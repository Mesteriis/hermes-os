use super::super::support::*;
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
    let owner_persona = crate::domains::persons::service::PersonCommandService::new(pool)
        .set_owner_persona_manual(&req.person_id)
        .await?;
    Ok(Json(OwnerPersonaResponse {
        owner_persona: Some(owner_persona),
    }))
}
