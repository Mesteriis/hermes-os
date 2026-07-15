//! Hidden WebView WhatsApp conversation command endpoints.

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;

use crate::app::api_support::stores::integration_stores::{
    whatsapp_provider_runtime_service, whatsapp_secret_reference_store,
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppConversationCommandRequest, WhatsAppProviderCommandResponse,
};

use super::{optional_string, publish_whatsapp_command_event, required_string};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppConversationCommandApiRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) idempotency_key: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) confirmation_decision: Option<String>,
    pub(crate) invite_link: Option<String>,
}

pub(crate) async fn post_whatsapp_conversation_mark_read(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_mark_read(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_mark_unread(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_mark_unread(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_archive(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_archive(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_unarchive(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_unarchive(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_mute(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_mute(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_unmute(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_unmute(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_pin(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_pin(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_unpin(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_unpin(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_join_group(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_join_group(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_conversation_leave_group(
    State(state): State<AppState>,
    Path(_conversation_id): Path<String>,
    Json(request): Json<WhatsAppConversationCommandApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_conversation_command_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_leave_group(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

fn validate_whatsapp_conversation_command_request(
    request: WhatsAppConversationCommandApiRequest,
) -> Result<WhatsAppConversationCommandRequest, ApiError> {
    Ok(WhatsAppConversationCommandRequest {
        command_id: request.command_id,
        idempotency_key: required_string("idempotency_key", &request.idempotency_key)?,
        account_id: required_string("account_id", &request.account_id)?,
        provider_chat_id: required_string("provider_chat_id", &request.provider_chat_id)?,
        confirmation_decision: optional_string(
            "confirmation_decision",
            request.confirmation_decision,
        )?,
        invite_link: optional_string("invite_link", request.invite_link)?,
    })
}
