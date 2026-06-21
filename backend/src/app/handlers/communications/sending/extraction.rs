use super::super::*;

pub(crate) async fn post_v1_extract_tasks(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let runtime_settings = ai_runtime_settings(&state).await?;
    let svc = crate::domains::communications::extract::EmailExtractService::new(ai_runtime_port(
        &state,
        &runtime_settings,
    ));
    let tasks = svc.extract_tasks(&msg).await?;
    Ok(Json(serde_json::json!({"tasks": tasks})))
}

pub(crate) async fn post_v1_extract_notes(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::domains::communications::extract::EmailExtractService::new(None);
    let notes = svc.extract_notes(&msg).await?;
    Ok(Json(serde_json::json!({"notes": notes})))
}
