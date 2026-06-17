use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use serde_json::json;

use crate::app::{ApiError, AppState};
use crate::domains::api_support::{api_audit_log, telegram_store};
use crate::integrations::telegram::client::{TelegramError, lifecycle};
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::events::bus::telegram_event_types;

use super::super::chat_actions::{TelegramChatActionRequest, TelegramChatActionResponse};
use super::super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
};
use super::build_event;

pub(crate) async fn post_telegram_message_mark_read(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.mark_read")
        .await?;
    let store = telegram_store(&state)?;
    let message = store.message_by_id(&message_id).await?.ok_or_else(|| {
        ApiError::Telegram(TelegramError::InvalidRequest(format!(
            "Telegram message `{message_id}` was not found"
        )))
    })?;
    let provider_chat_id = message.provider_chat_id.clone().ok_or_else(|| {
        ApiError::Telegram(TelegramError::InvalidRequest(
            "Telegram message does not include provider chat id".to_owned(),
        ))
    })?;
    if message.account_id != request.account_id {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "message account_id does not match mark-read request".to_owned(),
        )));
    }
    if provider_chat_id != request.provider_chat_id {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "message provider_chat_id does not match mark-read request".to_owned(),
        )));
    }

    let chat = store
        .telegram_chat(&message.account_id, &provider_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram chat projection for message `{message_id}` was not found"
            )))
        })?;

    store
        .set_chat_last_read_at(&chat.telegram_chat_id, Some(Utc::now()))
        .await?;
    store
        .apply_provider_unread_counts(
            &chat.telegram_chat_id,
            None,
            None,
            Some(&message.provider_message_id),
            "api.telegram.message.mark_read",
        )
        .await?;
    let metadata = store
        .recompute_chat_unread_count(&chat.telegram_chat_id)
        .await?;

    let command_id = lifecycle::new_command_id();
    let _command = lifecycle::insert_command(
        store.pool(),
        &command_id,
        &request.account_id,
        "mark_read",
        &format!(
            "mark_read:{}:{}",
            message.message_id,
            Utc::now().timestamp_millis()
        ),
        &provider_chat_id,
        Some(&message.provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        AUDIT_ACTOR_ID,
        json!({
            "source": "telegram_message_mark_read",
            "message_id": &message.message_id,
            "last_read_inbox_provider_message_id": &message.provider_message_id,
        }),
        json!({
            "message_id": &message.message_id,
            "telegram_chat_id": &chat.telegram_chat_id,
            "provider_chat_id": &provider_chat_id,
            "provider_message_id": &message.provider_message_id,
        }),
        json!({
            "source": "telegram_message_mark_read",
            "message_id": &message.message_id,
            "last_read_inbox_provider_message_id": &message.provider_message_id,
        }),
    )
    .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_message_mark_read(
            AUDIT_ACTOR_ID,
            &message.message_id,
            &request.account_id,
            &provider_chat_id,
            &message.provider_message_id,
        ))
        .await?;

    let command_event = build_event(
        telegram_event_types::COMMAND_STATUS_CHANGED,
        &request.account_id,
        &command_id,
        json!({
            "command_id": &command_id,
            "command_kind": "mark_read",
            "action": "mark_read",
            "provider_chat_id": &provider_chat_id,
            "telegram_chat_id": &chat.telegram_chat_id,
            "message_id": &message.provider_message_id,
            "status": "queued",
            "chat": &chat,
        }),
    );
    publish_telegram_event(&state, command_event).await?;

    let refreshed_chat = store.telegram_chat_by_id(&chat.telegram_chat_id).await?;
    let chat_updated_event = build_event(
        telegram_event_types::CHAT_UPDATED,
        &request.account_id,
        &chat.telegram_chat_id,
        json!({
            "provider_chat_id": &provider_chat_id,
            "telegram_chat_id": &chat.telegram_chat_id,
            "action": "mark_read",
            "chat": refreshed_chat,
        }),
    );
    publish_telegram_event(&state, chat_updated_event).await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id: chat.telegram_chat_id,
        action: "mark_read".to_owned(),
        status: "read".to_owned(),
        metadata,
    }))
}
