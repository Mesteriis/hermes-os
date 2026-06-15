use super::*;
use crate::domains::mail::ai_state::{
    MailAiStateRecord, MailAiStateStore, MailAiStateTransitionRequest,
};

pub(crate) async fn get_v1_message_ai_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<MailAiStateRecord>, ApiError> {
    let Some(record) = ai_state_store(&state)?.current(&message_id).await? else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(record))
}

pub(crate) async fn put_v1_message_ai_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<MailAiStateTransitionRequest>,
) -> Result<Json<MailAiStateRecord>, ApiError> {
    let Some(record) = ai_state_store(&state)?
        .transition(&message_id, request)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(record))
}

fn ai_state_store(state: &AppState) -> Result<MailAiStateStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(MailAiStateStore::new(pool))
}
