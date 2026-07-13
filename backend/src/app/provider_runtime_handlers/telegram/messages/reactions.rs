use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::Deserialize;

use crate::app::api_support::{
    automation_calls::*,
    communications::*,
    ensure_fixture_routes_enabled,
    messaging_integrations::*,
    platform_dtos::*,
    query_parsing::{communication::*, documents::*, graph::*, personas::*, projects::*, tasks::*},
    review_commands::*,
    review_lists::*,
    stores::{ai_runtime::*, domain_stores::*, integration_stores::*, settings_vault::*},
    telegram_capabilities::*,
    whatsapp_capabilities::*,
};
use crate::app::provider_runtime_handlers::whatsapp::messages::{
    delete_whatsapp_command_react, post_whatsapp_command_react,
};
use crate::app::{ApiError, AppState};
use crate::integrations::telegram::client::models::messages::{
    TelegramReactionListResponse, TelegramReactionRequest, TelegramReactionResponse,
};
use crate::integrations::telegram::client::telegram_self_provider_member_id;
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppProviderCommandResponse, WhatsAppReactionRequest,
};

use super::super::helpers::ensure_telegram_account_operation_allowed;
use super::telegram_message_write_service;

#[derive(Deserialize)]
pub(crate) struct TelegramReactionCommandRequest {
    account_id: String,
    provider_chat_id: String,
    provider_message_id: String,
    reaction_emoji: String,
    sender_id: Option<String>,
    sender_display_name: Option<String>,
    command_id: Option<String>,
}

/// POST /api/v1/communications/messages/{message_id}/reactions
pub(crate) async fn post_telegram_reaction(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramReactionCommandRequest>,
) -> Result<Json<TelegramReactionResponse>, ApiError> {
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };
    let request = normalize_reaction_request(
        request,
        account.provider_kind.is_telegram(),
        &account.external_account_id,
    )?;

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
    let request = TelegramReactionCommandRequest {
        account_id: query.account_id.clone(),
        provider_chat_id: query.provider_chat_id.clone(),
        provider_message_id: query.provider_message_id.clone(),
        reaction_emoji: query.reaction_emoji.clone(),
        sender_id: query.sender_id.clone(),
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
    let request = normalize_reaction_request(
        request,
        account.provider_kind.is_telegram(),
        &account.external_account_id,
    )?;

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

fn normalize_reaction_request(
    request: TelegramReactionCommandRequest,
    is_telegram: bool,
    external_account_id: &str,
) -> Result<TelegramReactionRequest, ApiError> {
    let sender_id = request
        .sender_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            if is_telegram {
                telegram_self_provider_member_id(external_account_id)
            } else {
                let value = external_account_id.trim();
                (!value.is_empty()).then(|| value.to_owned())
            }
        })
        .ok_or(ApiError::InvalidCommunicationQuery(
            "reaction sender identity is unavailable for this provider account",
        ))?;

    Ok(TelegramReactionRequest {
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_message_id: request.provider_message_id,
        reaction_emoji: request.reaction_emoji,
        sender_id,
        sender_display_name: request.sender_display_name,
        command_id: request.command_id,
    })
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

#[cfg(test)]
mod tests {
    use super::{TelegramReactionCommandRequest, normalize_reaction_request};

    fn request(sender_id: Option<&str>) -> TelegramReactionCommandRequest {
        TelegramReactionCommandRequest {
            account_id: "telegram-account".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            provider_message_id: "message-1".to_owned(),
            reaction_emoji: "👍".to_owned(),
            sender_id: sender_id.map(ToOwned::to_owned),
            sender_display_name: None,
            command_id: None,
        }
    }

    #[test]
    fn derives_telegram_reaction_sender_from_account_identity() {
        let normalized = normalize_reaction_request(request(None), true, "telegram:42")
            .expect("normalized Telegram reaction request");

        assert_eq!(normalized.sender_id, "user:42");
    }

    #[test]
    fn preserves_explicit_reaction_sender_identity() {
        let normalized = normalize_reaction_request(request(Some("user:9")), true, "telegram:42")
            .expect("normalized Telegram reaction request");

        assert_eq!(normalized.sender_id, "user:9");
    }
}
