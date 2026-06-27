use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;

use crate::app::api_support::{TelegramReactionDeleteQuery, communication_provider_account_store};
use crate::app::provider_runtime_handlers::whatsapp::{
    delete_whatsapp_command_react, post_whatsapp_command_react,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramReactionListResponse, TelegramReactionRequest, TelegramReactionResponse,
    WhatsAppProviderCommandResponse, WhatsAppReactionRequest,
};

use super::super::helpers::ensure_telegram_account_operation_allowed;
use super::telegram_message_write_service;

/// POST /api/v1/communications/messages/{message_id}/reactions
pub(crate) async fn post_telegram_reaction(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramReactionRequest>,
) -> Result<Json<TelegramReactionResponse>, ApiError> {
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(next_whatsapp_command_id);
        let response = post_whatsapp_command_react(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppReactionRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("react", &command_id),
                account_id: request.account_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                provider_message_id: request.provider_message_id.clone(),
                reaction_emoji: request.reaction_emoji.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_reaction_response(
            &message_id,
            &request,
            true,
            &response,
        )));
    }

    ensure_telegram_account_operation_allowed(&state, &request.account_id, "reactions.add").await?;
    let response = telegram_message_write_service(&state)?
        .add_reaction(&message_id, request)
        .await?;
    Ok(Json(response))
}

/// DELETE /api/v1/communications/messages/{message_id}/reactions
pub(crate) async fn delete_telegram_reaction(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<TelegramReactionDeleteQuery>,
) -> Result<Json<TelegramReactionResponse>, ApiError> {
    let request = TelegramReactionRequest {
        account_id: query.account_id.clone(),
        provider_chat_id: query.provider_chat_id.clone(),
        provider_message_id: query.provider_message_id.clone(),
        reaction_emoji: query.reaction_emoji.clone(),
        sender_id: query.sender_id.clone().unwrap_or_default(),
        sender_display_name: query.sender_display_name.clone(),
        command_id: query.command_id.clone(),
    };
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(next_whatsapp_command_id);
        let response = delete_whatsapp_command_react(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppReactionRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("unreact", &command_id),
                account_id: request.account_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                provider_message_id: request.provider_message_id.clone(),
                reaction_emoji: request.reaction_emoji.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_reaction_response(
            &message_id,
            &request,
            false,
            &response,
        )));
    }

    ensure_telegram_account_operation_allowed(&state, &request.account_id, "reactions.remove")
        .await?;
    let response = telegram_message_write_service(&state)?
        .remove_reaction(&message_id, request)
        .await?;
    Ok(Json(response))
}

/// GET /api/v1/communications/messages/{message_id}/reactions
pub(crate) async fn get_telegram_reactions(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramReactionListResponse>, ApiError> {
    let response = telegram_message_write_service(&state)?
        .reactions(&message_id)
        .await?;
    Ok(Json(response))
}

fn next_whatsapp_command_id() -> String {
    format!(
        "whatsapp-command-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

fn whatsapp_command_idempotency_key(operation: &str, command_id: &str) -> String {
    format!("communications:whatsapp:{operation}:{command_id}")
}

fn whatsapp_command_response_to_reaction_response(
    message_id: &str,
    request: &TelegramReactionRequest,
    is_active: bool,
    response: &WhatsAppProviderCommandResponse,
) -> TelegramReactionResponse {
    TelegramReactionResponse {
        reaction_id: format!(
            "{}:{}:{}",
            request.provider_message_id, request.reaction_emoji, response.command_id
        ),
        message_id: message_id.to_owned(),
        account_id: response.account_id.clone(),
        provider_chat_id: response.provider_chat_id.clone(),
        provider_message_id: response.provider_message_id.clone().unwrap_or_default(),
        reaction_emoji: request.reaction_emoji.clone(),
        is_active,
        status: response.status.clone(),
        timestamp: response.updated_at,
    }
}
