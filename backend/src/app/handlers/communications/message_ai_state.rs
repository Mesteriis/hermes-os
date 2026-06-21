use super::*;
use crate::domains::communications::ai_state::{
    CommunicationAiStateRecord, CommunicationAiStateStore, CommunicationAiStateTransitionRequest,
};
use crate::domains::communications::service::CommunicationCommandService;

pub(crate) async fn get_v1_message_ai_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<CommunicationAiStateRecord>, ApiError> {
    let Some(record) = ai_state_store(&state)?.current(&message_id).await? else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(record))
}

pub(crate) async fn put_v1_message_ai_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<CommunicationAiStateTransitionRequest>,
) -> Result<Json<CommunicationAiStateRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let record = CommunicationCommandService::new(pool)
        .transition_message_ai_state(&message_id, request)
        .await?;
    Ok(Json(record))
}

fn ai_state_store(state: &AppState) -> Result<CommunicationAiStateStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(crate::app::api_support::app_store::<
        CommunicationAiStateStore,
    >(pool))
}
