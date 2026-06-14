use super::super::*;

pub(crate) async fn post_v1_imap_mark_read(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    store
        .transition_workflow_state(&message_id, WorkflowState::Reviewed)
        .await?;
    Ok(Json(serde_json::json!({"marked_read": true})))
}

pub(crate) async fn post_v1_imap_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let updated = store
        .move_to_local_trash(&message_id, "imap-delete-alias")
        .await?;
    Ok(Json(serde_json::json!({
        "deleted": true,
        "provider_deleted": false,
        "local_state": updated.local_state.as_str()
    })))
}

pub(crate) async fn post_v1_message_trash(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let updated = message_store(&state)?
        .move_to_local_trash(&message_id, "user_deleted")
        .await?;
    Ok(Json(serde_json::json!({
        "message_id": updated.message_id,
        "local_state": updated.local_state.as_str(),
        "provider_deleted": false
    })))
}

pub(crate) async fn post_v1_message_restore(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let updated = message_store(&state)?
        .restore_from_local_trash(&message_id)
        .await?;
    Ok(Json(serde_json::json!({
        "message_id": updated.message_id,
        "local_state": updated.local_state.as_str()
    })))
}
