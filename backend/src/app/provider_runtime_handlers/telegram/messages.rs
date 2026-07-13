use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;

use super::helpers::ensure_telegram_account_operation_allowed;
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
use crate::app::provider_runtime_handlers::telegram::chats::canonical_communication_conversation;
use crate::app::provider_runtime_handlers::whatsapp::{
    WhatsAppConversationCommandApiRequest, post_whatsapp_command_delete,
    post_whatsapp_command_edit, post_whatsapp_command_send_text,
    post_whatsapp_conversation_archive, post_whatsapp_conversation_mark_read,
    post_whatsapp_conversation_mark_unread, post_whatsapp_conversation_mute,
    post_whatsapp_conversation_pin, post_whatsapp_conversation_unarchive,
    post_whatsapp_conversation_unmute, post_whatsapp_conversation_unpin,
};
use crate::app::{ApiError, AppState};
use crate::application::communication_provider_writes::{
    CommunicationConversationMessageRequest, CommunicationProviderMessageCommandResponse,
};
use crate::application::telegram_runtime;
use crate::integrations::telegram::client::models::messages::{
    TelegramDeleteRequest, TelegramEditRequest, TelegramForwardRequest, TelegramLifecycleResponse,
    TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessageTombstoneListResponse,
    TelegramMessageVersionListResponse, TelegramPinRequest, TelegramReplyRequest,
    TelegramRestoreVisibilityRequest,
};
use crate::integrations::telegram::client::{NewTelegramMessage, TelegramMessageIngestResult};
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppDeleteRequest, WhatsAppEditRequest, WhatsAppProviderCommandResponse,
    WhatsAppTextSendRequest,
};

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
        let response = post_whatsapp_command_send_text(
            State(state.clone()),
            Json(WhatsAppTextSendRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("send_text", &command_id),
                account_id: request.account_id.clone(),
                provider_chat_id: conversation_id.clone(),
                text: request.text.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_communication_response(
            &command_id,
            &conversation_id,
            None,
            &response,
        )));
    }

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
    Json(payload): Json<Value>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    let request: ProviderNeutralEditRequest = serde_json::from_value(payload)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid message edit payload"))?;
    let command_id = request
        .command_id
        .clone()
        .unwrap_or_else(next_whatsapp_command_id);
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let response = post_whatsapp_command_edit(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppEditRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("edit", &command_id),
                account_id: request.account_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                provider_message_id: request.provider_message_id.clone(),
                text: request.new_text.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_lifecycle_response(
            "edit",
            &message_id,
            &response,
        )));
    }

    let request = TelegramEditRequest {
        command_id,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_message_id: request.provider_message_id,
        new_text: request.new_text,
    };
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "messages.edit").await?;
    let response = telegram_message_write_service(&state)?
        .edit_message(&message_id, &request)
        .await?;
    Ok(Json(response))
}

pub(crate) async fn post_telegram_message_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<TelegramLifecycleResponse>, ApiError> {
    let request: ProviderNeutralDeleteRequest = serde_json::from_value(payload)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid message delete payload"))?;
    let command_id = request
        .command_id
        .clone()
        .unwrap_or_else(next_whatsapp_command_id);
    let Some(account) = communication_provider_account_store(&state)?
        .get(&request.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if account.provider_kind.is_whatsapp() {
        let response = post_whatsapp_command_delete(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppDeleteRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("delete", &command_id),
                confirmation_decision: Some("confirmed".to_owned()),
                account_id: request.account_id.clone(),
                provider_chat_id: request.provider_chat_id.clone(),
                provider_message_id: request.provider_message_id.clone(),
            }),
        )
        .await?
        .0;
        return Ok(Json(whatsapp_command_response_to_lifecycle_response(
            "delete",
            &message_id,
            &response,
        )));
    }

    let request = TelegramDeleteRequest {
        command_id,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_message_id: request.provider_message_id,
        reason_class: request
            .reason_class
            .unwrap_or_else(|| "deleted_by_owner".to_owned()),
        actor_class: request.actor_class.unwrap_or_else(|| "owner".to_owned()),
        is_provider_delete: request.is_provider_delete.unwrap_or(false),
    };
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

pub(crate) async fn post_communication_conversation_pin(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&state, &conversation_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider conversation was not found",
        ))?;
    let Some(account) = communication_provider_account_store(&state)?
        .get(&conversation.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if !account.provider_kind.is_whatsapp() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider-neutral conversation pin currently supports WhatsApp conversations only",
        ));
    }

    let command_id = next_whatsapp_command_id();
    let response = post_whatsapp_conversation_pin(
        State(state),
        Path(conversation.provider_chat_id.clone()),
        Json(WhatsAppConversationCommandApiRequest {
            command_id: Some(command_id.clone()),
            idempotency_key: whatsapp_command_idempotency_key("pin", &command_id),
            account_id: conversation.account_id.clone(),
            provider_chat_id: conversation.provider_chat_id.clone(),
            confirmation_decision: Some("confirmed".to_owned()),
            invite_link: None,
        }),
    )
    .await?
    .0;

    Ok(Json(
        whatsapp_conversation_command_response_to_communication_response(
            &conversation_id,
            "pin",
            true,
            &response,
        ),
    ))
}

pub(crate) async fn post_communication_conversation_unpin(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&state, &conversation_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider conversation was not found",
        ))?;
    let Some(account) = communication_provider_account_store(&state)?
        .get(&conversation.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if !account.provider_kind.is_whatsapp() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider-neutral conversation unpin currently supports WhatsApp conversations only",
        ));
    }

    let command_id = next_whatsapp_command_id();
    let response = post_whatsapp_conversation_unpin(
        State(state),
        Path(conversation.provider_chat_id.clone()),
        Json(WhatsAppConversationCommandApiRequest {
            command_id: Some(command_id.clone()),
            idempotency_key: whatsapp_command_idempotency_key("unpin", &command_id),
            account_id: conversation.account_id.clone(),
            provider_chat_id: conversation.provider_chat_id.clone(),
            confirmation_decision: Some("confirmed".to_owned()),
            invite_link: None,
        }),
    )
    .await?
    .0;

    Ok(Json(
        whatsapp_conversation_command_response_to_communication_response(
            &conversation_id,
            "unpin",
            false,
            &response,
        ),
    ))
}

pub(crate) async fn post_communication_conversation_archive(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&state, &conversation_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider conversation was not found",
        ))?;
    let Some(account) = communication_provider_account_store(&state)?
        .get(&conversation.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if !account.provider_kind.is_whatsapp() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider-neutral conversation archive currently supports WhatsApp conversations only",
        ));
    }

    let command_id = next_whatsapp_command_id();
    let response = post_whatsapp_conversation_archive(
        State(state),
        Path(conversation.provider_chat_id.clone()),
        Json(WhatsAppConversationCommandApiRequest {
            command_id: Some(command_id.clone()),
            idempotency_key: whatsapp_command_idempotency_key("archive", &command_id),
            account_id: conversation.account_id.clone(),
            provider_chat_id: conversation.provider_chat_id.clone(),
            confirmation_decision: Some("confirmed".to_owned()),
            invite_link: None,
        }),
    )
    .await?
    .0;

    Ok(Json(
        whatsapp_conversation_command_response_to_communication_response(
            &conversation_id,
            "archive",
            true,
            &response,
        ),
    ))
}

pub(crate) async fn post_communication_conversation_unarchive(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&state, &conversation_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider conversation was not found",
        ))?;
    let Some(account) = communication_provider_account_store(&state)?
        .get(&conversation.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if !account.provider_kind.is_whatsapp() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider-neutral conversation unarchive currently supports WhatsApp conversations only",
        ));
    }

    let command_id = next_whatsapp_command_id();
    let response = post_whatsapp_conversation_unarchive(
        State(state),
        Path(conversation.provider_chat_id.clone()),
        Json(WhatsAppConversationCommandApiRequest {
            command_id: Some(command_id.clone()),
            idempotency_key: whatsapp_command_idempotency_key("unarchive", &command_id),
            account_id: conversation.account_id.clone(),
            provider_chat_id: conversation.provider_chat_id.clone(),
            confirmation_decision: Some("confirmed".to_owned()),
            invite_link: None,
        }),
    )
    .await?
    .0;

    Ok(Json(
        whatsapp_conversation_command_response_to_communication_response(
            &conversation_id,
            "unarchive",
            false,
            &response,
        ),
    ))
}

pub(crate) async fn post_communication_conversation_mute(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&state, &conversation_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider conversation was not found",
        ))?;
    let Some(account) = communication_provider_account_store(&state)?
        .get(&conversation.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if !account.provider_kind.is_whatsapp() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider-neutral conversation mute currently supports WhatsApp conversations only",
        ));
    }

    let command_id = next_whatsapp_command_id();
    let response = post_whatsapp_conversation_mute(
        State(state),
        Path(conversation.provider_chat_id.clone()),
        Json(WhatsAppConversationCommandApiRequest {
            command_id: Some(command_id.clone()),
            idempotency_key: whatsapp_command_idempotency_key("mute", &command_id),
            account_id: conversation.account_id.clone(),
            provider_chat_id: conversation.provider_chat_id.clone(),
            confirmation_decision: Some("confirmed".to_owned()),
            invite_link: None,
        }),
    )
    .await?
    .0;

    Ok(Json(
        whatsapp_conversation_command_response_to_communication_response(
            &conversation_id,
            "mute",
            true,
            &response,
        ),
    ))
}

pub(crate) async fn post_communication_conversation_unmute(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&state, &conversation_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider conversation was not found",
        ))?;
    let Some(account) = communication_provider_account_store(&state)?
        .get(&conversation.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if !account.provider_kind.is_whatsapp() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider-neutral conversation unmute currently supports WhatsApp conversations only",
        ));
    }

    let command_id = next_whatsapp_command_id();
    let response = post_whatsapp_conversation_unmute(
        State(state),
        Path(conversation.provider_chat_id.clone()),
        Json(WhatsAppConversationCommandApiRequest {
            command_id: Some(command_id.clone()),
            idempotency_key: whatsapp_command_idempotency_key("unmute", &command_id),
            account_id: conversation.account_id.clone(),
            provider_chat_id: conversation.provider_chat_id.clone(),
            confirmation_decision: Some("confirmed".to_owned()),
            invite_link: None,
        }),
    )
    .await?
    .0;

    Ok(Json(
        whatsapp_conversation_command_response_to_communication_response(
            &conversation_id,
            "unmute",
            false,
            &response,
        ),
    ))
}

pub(crate) async fn post_communication_conversation_mark_read(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&state, &conversation_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider conversation was not found",
        ))?;
    let Some(account) = communication_provider_account_store(&state)?
        .get(&conversation.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if !account.provider_kind.is_whatsapp() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider-neutral conversation mark-read currently supports WhatsApp conversations only",
        ));
    }

    let command_id = next_whatsapp_command_id();
    let response = post_whatsapp_conversation_mark_read(
        State(state),
        Path(conversation.provider_chat_id.clone()),
        Json(WhatsAppConversationCommandApiRequest {
            command_id: Some(command_id.clone()),
            idempotency_key: whatsapp_command_idempotency_key("mark_read", &command_id),
            account_id: conversation.account_id.clone(),
            provider_chat_id: conversation.provider_chat_id.clone(),
            confirmation_decision: Some("confirmed".to_owned()),
            invite_link: None,
        }),
    )
    .await?
    .0;

    Ok(Json(
        whatsapp_conversation_command_response_to_communication_response(
            &conversation_id,
            "mark_read",
            false,
            &response,
        ),
    ))
}

pub(crate) async fn post_communication_conversation_mark_unread(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<CommunicationProviderConversationCommandResponse>, ApiError> {
    let conversation = canonical_communication_conversation(&state, &conversation_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "provider conversation was not found",
        ))?;
    let Some(account) = communication_provider_account_store(&state)?
        .get(&conversation.account_id)
        .await?
    else {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider account was not found",
        ));
    };

    if !account.provider_kind.is_whatsapp() {
        return Err(ApiError::InvalidCommunicationQuery(
            "provider-neutral conversation mark-unread currently supports WhatsApp conversations only",
        ));
    }

    let command_id = next_whatsapp_command_id();
    let response = post_whatsapp_conversation_mark_unread(
        State(state),
        Path(conversation.provider_chat_id.clone()),
        Json(WhatsAppConversationCommandApiRequest {
            command_id: Some(command_id.clone()),
            idempotency_key: whatsapp_command_idempotency_key("mark_unread", &command_id),
            account_id: conversation.account_id.clone(),
            provider_chat_id: conversation.provider_chat_id.clone(),
            confirmation_decision: Some("confirmed".to_owned()),
            invite_link: None,
        }),
    )
    .await?
    .0;

    Ok(Json(
        whatsapp_conversation_command_response_to_communication_response(
            &conversation_id,
            "mark_unread",
            true,
            &response,
        ),
    ))
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

use crate::integrations::telegram::client::models::messages::{
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

#[derive(Clone, Debug, Deserialize)]
struct ProviderNeutralEditRequest {
    command_id: Option<String>,
    account_id: String,
    provider_chat_id: String,
    provider_message_id: String,
    new_text: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ProviderNeutralDeleteRequest {
    command_id: Option<String>,
    account_id: String,
    provider_chat_id: String,
    provider_message_id: String,
    reason_class: Option<String>,
    actor_class: Option<String>,
    is_provider_delete: Option<bool>,
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

fn whatsapp_command_response_to_communication_response(
    command_id: &str,
    conversation_id: &str,
    message_id: Option<&str>,
    response: &WhatsAppProviderCommandResponse,
) -> CommunicationProviderMessageCommandResponse {
    CommunicationProviderMessageCommandResponse {
        message_id: message_id.unwrap_or(command_id).to_owned(),
        raw_record_id: String::new(),
        conversation_id: conversation_id.to_owned(),
        provider_chat_id: response.provider_chat_id.clone(),
        provider_message_id: response.provider_message_id.clone(),
        channel_kind: if response.provider_kind == "whatsapp_business_cloud" {
            "whatsapp_business_cloud"
        } else {
            "whatsapp_web"
        },
        status: "queued".to_owned(),
        command_id: response.command_id.clone(),
        provider: "whatsapp",
    }
}

fn whatsapp_command_response_to_lifecycle_response(
    operation: &str,
    message_id: &str,
    response: &WhatsAppProviderCommandResponse,
) -> TelegramLifecycleResponse {
    TelegramLifecycleResponse {
        operation: operation.to_owned(),
        message_id: message_id.to_owned(),
        account_id: response.account_id.clone(),
        provider_chat_id: response.provider_chat_id.clone(),
        provider_message_id: response.provider_message_id.clone().unwrap_or_default(),
        status: "queued".to_owned(),
        timestamp: response.updated_at,
        version_number: None,
        tombstone_id: None,
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct CommunicationProviderConversationCommandResponse {
    conversation_id: String,
    provider_chat_id: String,
    channel_kind: String,
    action: String,
    status: String,
    command_id: String,
    provider: &'static str,
    active: bool,
}

fn whatsapp_conversation_command_response_to_communication_response(
    conversation_id: &str,
    action: &str,
    active: bool,
    response: &WhatsAppProviderCommandResponse,
) -> CommunicationProviderConversationCommandResponse {
    CommunicationProviderConversationCommandResponse {
        conversation_id: conversation_id.to_owned(),
        provider_chat_id: response.provider_chat_id.clone(),
        channel_kind: if response.provider_kind == "whatsapp_business_cloud" {
            "whatsapp_business_cloud".to_owned()
        } else {
            "whatsapp_web".to_owned()
        },
        action: action.to_owned(),
        status: "queued".to_owned(),
        command_id: response.command_id.clone(),
        provider: "whatsapp",
        active,
    }
}
