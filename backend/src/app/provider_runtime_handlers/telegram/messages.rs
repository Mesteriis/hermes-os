use axum::Json;
use axum::extract::{Path, State};

use super::helpers::ensure_telegram_account_operation_allowed;
use crate::app::api_support::{
    ensure_fixture_routes_enabled, telegram_fixture_ingest_service, telegram_message_write_service,
    telegram_runtime_use_case_context,
};
use crate::app::{ApiError, AppState};
use crate::application::communication_provider_writes::{
    CommunicationConversationMessageRequest, CommunicationProviderMessageCommandResponse,
};
use crate::application::provider_runtime_contracts::NewTelegramMessage;
use crate::application::provider_runtime_contracts::{
    TelegramDeleteRequest, TelegramEditRequest, TelegramForwardRequest, TelegramLifecycleResponse,
    TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessageIngestResult,
    TelegramMessageTombstoneListResponse, TelegramMessageVersionListResponse, TelegramPinRequest,
    TelegramReplyRequest, TelegramRestoreVisibilityRequest,
};
use crate::application::telegram_runtime;

mod mark_read;
mod reactions;

pub(crate) use mark_read::post_telegram_message_mark_read;
pub(crate) use reactions::{
    delete_telegram_reaction, get_telegram_reactions, post_telegram_reaction,
};

pub(crate) async fn post_telegram_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewTelegramMessage>,
) -> Result<Json<TelegramMessageIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let response = telegram_fixture_ingest_service(&state)?
        .ingest_message(&request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_manual_send(
    State(state): State<AppState>,
    Json(request): Json<TelegramManualSendRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.send_text")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_manual_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_communication_conversation_message(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(mut request): Json<CommunicationConversationMessageRequest>,
) -> Result<Json<CommunicationProviderMessageCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.send_text")
        .await?;
    let command_id = request
        .command_id
        .clone()
        .unwrap_or_else(crate::application::communication_provider_writes::new_telegram_command_id);
    request.command_id = Some(command_id.clone());
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_conversation_message(&runtime_context, &conversation_id, request)
        .await?;
    Ok(Json(CommunicationProviderMessageCommandResponse::telegram(
        command_id, &response,
    )))
}

pub(crate) async fn post_telegram_message_reply(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<TelegramReplyRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.reply")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_reply_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_forward(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<TelegramForwardRequest>,
) -> Result<Json<TelegramManualSendResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.forward")
        .await?;
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = telegram_message_write_service(&state)?
        .send_forward_message(&runtime_context, &request)
        .await?;
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// Lifecycle endpoints (ADR-0091)
// ---------------------------------------------------------------------------

pub(crate) async fn post_telegram_message_edit(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramEditRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.edit").await?;
    let response = telegram_message_write_service(&state)?
        .edit_message(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramDeleteRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.delete")
        .await?;
    let response = telegram_message_write_service(&state)?
        .delete_message(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_restore_visibility(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramRestoreVisibilityRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(
        &state,
        &request.account_id,
        "messages.restore_visibility",
    )
    .await?;
    let response = telegram_message_write_service(&state)?
        .restore_message_visibility(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_pin(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramPinRequest>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.pin").await?;
    let response = telegram_message_write_service(&state)?
        .pin_message(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn get_telegram_message_versions(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramMessageVersionListResponse>, ApiError> {
    let response = telegram_message_write_service(&state)?
        .message_versions(&message_id)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn get_telegram_message_tombstones(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramMessageTombstoneListResponse>, ApiError> {
    let response = telegram_message_write_service(&state)?
        .message_tombstones(&message_id)
        .await?;
    Ok(Json(response))
}

use crate::application::provider_runtime_contracts::{
    TelegramForwardChainResponse, TelegramReplyChainResponse,
};

/// GET /api/v1/communications/messages/{message_id}/reply-chain
pub(crate) async fn get_telegram_reply_chain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramReplyChainResponse>, ApiError> {
    let response = telegram_message_write_service(&state)?
        .reply_chain(&message_id)
        .await?;
    Ok(Json(response))
}

/// GET /api/v1/communications/messages/{message_id}/forward-chain
pub(crate) async fn get_telegram_forward_chain(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramForwardChainResponse>, ApiError> {
    let response = telegram_message_write_service(&state)?
        .forward_chain(&message_id)
        .await?;
    Ok(Json(response))
}
