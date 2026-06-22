use axum::Json;
use axum::extract::{Path, Query, State};

use crate::app::api_support::TelegramReactionDeleteQuery;
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    TelegramReactionListResponse, TelegramReactionRequest, TelegramReactionResponse,
};

use super::super::helpers::ensure_telegram_account_operation_allowed;
use super::telegram_message_write_service;

/// POST /api/v1/communications/messages/{message_id}/reactions
pub(crate) async fn post_telegram_reaction(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramReactionRequest>,
) -> Result<Json<TelegramReactionResponse>, ApiError> {
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
        sender_id: query.sender_id.clone(),
        sender_display_name: query.sender_display_name.clone(),
        command_id: query.command_id.clone(),
    };
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
