use super::super::*;

#[derive(Serialize)]
pub(crate) struct PersonaListResponse {
    pub(super) items: Vec<crate::domains::communications::personas::CommunicationPersona>,
}

#[derive(Deserialize)]
pub(crate) struct NewPersonaRequest {
    pub(super) persona_id: String,
    pub(super) name: String,
    pub(super) account_id: String,
    pub(super) display_name: String,
    pub(super) signature: Option<String>,
    pub(super) default_language: Option<String>,
    pub(super) default_tone: Option<String>,
    pub(super) is_default: Option<bool>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn get_v1_personas(
    State(state): State<AppState>,
) -> Result<Json<PersonaListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::communications::personas::CommunicationPersonaStore::new(pool);
    let items = store.list().await?;
    Ok(Json(PersonaListResponse { items }))
}

pub(crate) async fn post_v1_persona(
    State(state): State<AppState>,
    Json(request): Json<NewPersonaRequest>,
) -> Result<Json<crate::domains::communications::personas::CommunicationPersona>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::domains::communications::personas::CommunicationPersonaStore::new(pool);
    let persona = store
        .upsert(
            &crate::domains::communications::personas::NewCommunicationPersona {
                persona_id: request.persona_id,
                name: request.name,
                account_id: request.account_id,
                display_name: request.display_name,
                signature: request.signature.unwrap_or_default(),
                default_language: request.default_language,
                default_tone: request.default_tone,
                is_default: request.is_default.unwrap_or(false),
                metadata: request.metadata.unwrap_or(serde_json::json!({})),
            },
        )
        .await?;
    Ok(Json(persona))
}
