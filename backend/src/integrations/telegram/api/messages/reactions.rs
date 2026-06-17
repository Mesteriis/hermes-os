use axum::Json;
use axum::extract::{Path, Query, State};
use serde_json::json;

use super::{AUDIT_ACTOR_ID, build_command_event, build_event};
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{TelegramReactionDeleteQuery, api_audit_log, telegram_store};
use crate::integrations::telegram::api::helpers::{
    ensure_telegram_account_operation_allowed, publish_telegram_event,
};
use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::{
    TelegramReactionListResponse, TelegramReactionRequest, TelegramReactionResponse,
};
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::bus::telegram_event_types;

/// POST /api/v1/telegram/messages/{message_id}/reactions
pub(crate) async fn post_telegram_reaction(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(mut request): Json<TelegramReactionRequest>,
) -> Result<Json<TelegramReactionResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "reactions.add").await?;
    let command_id = request
        .command_id
        .clone()
        .unwrap_or_else(lifecycle::new_command_id);
    request.command_id = Some(command_id.clone());
    let store = telegram_store(&state)?;
    let response = lifecycle::add_reaction(store.pool(), &request, &message_id).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_reaction(
            AUDIT_ACTOR_ID,
            &message_id,
            &request.account_id,
            &request.provider_chat_id,
            &request.reaction_emoji,
            true,
        ))
        .await?;

    let event = build_event(
        telegram_event_types::REACTION_CHANGED,
        &request.account_id,
        &message_id,
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "reaction_emoji": &request.reaction_emoji,
            "is_active": true,
        }),
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &request.account_id,
        &command_id,
        "react",
        &request.provider_chat_id,
        Some(&message_id),
        Some(&request.provider_message_id),
        "queued",
        json!({
            "telegram_message_id": &message_id,
            "reaction_emoji": &request.reaction_emoji,
            "sender_id": &request.sender_id,
            "sender_display_name": &request.sender_display_name,
            "is_active": true,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

/// DELETE /api/v1/telegram/messages/{message_id}/reactions
pub(crate) async fn delete_telegram_reaction(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<TelegramReactionDeleteQuery>,
) -> Result<Json<TelegramReactionResponse>, ApiError> {
    let command_id = query
        .command_id
        .clone()
        .unwrap_or_else(lifecycle::new_command_id);
    let request = TelegramReactionRequest {
        account_id: query.account_id.clone(),
        provider_chat_id: query.provider_chat_id.clone(),
        provider_message_id: query.provider_message_id.clone(),
        reaction_emoji: query.reaction_emoji.clone(),
        sender_id: query.sender_id.clone(),
        sender_display_name: query.sender_display_name.clone(),
        command_id: Some(command_id.clone()),
    };
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "reactions.remove")
        .await?;
    let store = telegram_store(&state)?;
    let response = lifecycle::remove_reaction(store.pool(), &request, &message_id).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_reaction(
            AUDIT_ACTOR_ID,
            &message_id,
            &request.account_id,
            &request.provider_chat_id,
            &request.reaction_emoji,
            false,
        ))
        .await?;

    let event = build_event(
        telegram_event_types::REACTION_CHANGED,
        &request.account_id,
        &message_id,
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "reaction_emoji": &request.reaction_emoji,
            "is_active": false,
        }),
    );
    publish_telegram_event(&state, event).await?;

    let command_event = build_command_event(
        &request.account_id,
        &command_id,
        "unreact",
        &request.provider_chat_id,
        Some(&message_id),
        Some(&request.provider_message_id),
        "queued",
        json!({
            "telegram_message_id": &message_id,
            "reaction_emoji": &request.reaction_emoji,
            "sender_id": &request.sender_id,
            "sender_display_name": &request.sender_display_name,
            "is_active": false,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    Ok(Json(response))
}

/// GET /api/v1/telegram/messages/{message_id}/reactions
pub(crate) async fn get_telegram_reactions(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<TelegramReactionListResponse>, ApiError> {
    let store = telegram_store(&state)?;
    let reactions = lifecycle::list_reactions(store.pool(), &message_id).await?;
    let summary = lifecycle::reaction_summary(store.pool(), &message_id).await?;
    Ok(Json(TelegramReactionListResponse {
        message_id,
        reactions,
        summary,
    }))
}
