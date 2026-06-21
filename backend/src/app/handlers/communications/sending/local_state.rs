use super::super::*;
use crate::domains::communications::service::CommunicationCommandService;

pub(crate) async fn post_v1_imap_mark_read(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationCommandService::new(pool)
        .mark_message_imap_read(&message_id)
        .await?;
    Ok(Json(serde_json::json!({"marked_read": true})))
}

pub(crate) async fn post_v1_imap_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let updated = CommunicationCommandService::new(pool)
        .move_message_to_local_trash(&message_id, "imap_delete_alias", "imap-delete-alias")
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
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let updated = CommunicationCommandService::new(pool)
        .move_message_to_local_trash(&message_id, "message_trash", "user_deleted")
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
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let updated = CommunicationCommandService::new(pool)
        .restore_message_from_local_trash(&message_id)
        .await?;
    Ok(Json(serde_json::json!({
        "message_id": updated.message_id,
        "local_state": updated.local_state.as_str()
    })))
}
