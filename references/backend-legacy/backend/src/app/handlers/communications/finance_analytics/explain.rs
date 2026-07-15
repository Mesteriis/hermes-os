use super::super::*;

#[derive(Serialize)]
pub(crate) struct MessageExplainResponse {
    pub(super) reasons: Vec<String>,
}

pub(crate) async fn get_v1_message_explain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<MessageExplainResponse>, ApiError> {
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let ctx = crate::domains::communications::explain::explain_importance(&message);
    Ok(Json(MessageExplainResponse {
        reasons: ctx.reasons,
    }))
}

#[derive(Serialize)]
pub(crate) struct SmartCcResponse {
    pub(super) suggestions: Vec<String>,
}

pub(crate) async fn get_v1_message_smart_cc(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<SmartCcResponse>, ApiError> {
    let store = message_store(&state)?;
    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let suggestions = crate::domains::communications::explain::smart_cc_suggestions(&message);
    Ok(Json(SmartCcResponse { suggestions }))
}
