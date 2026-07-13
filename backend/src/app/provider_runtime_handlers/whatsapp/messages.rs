//! Hidden WebView WhatsApp message command endpoints.

use axum::Json;
use axum::extract::{Path, State};

use crate::app::api_support::stores::{
    domain_stores::message_store,
    integration_stores::{whatsapp_provider_runtime_service, whatsapp_secret_reference_store},
};
use crate::app::{ApiError, AppState};
use crate::domains::communications::messages::ProviderChannelMessageStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppDeleteRequest, WhatsAppEditRequest, WhatsAppForwardRequest,
    WhatsAppProviderCommandResponse, WhatsAppReactionRequest, WhatsAppReplyRequest,
    WhatsAppTextSendRequest,
};

use super::publish_whatsapp_command_event;

pub(crate) async fn post_whatsapp_command_send_text(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppTextSendRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let response = whatsapp_provider_runtime_service(&state)?
        .request_send_text(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_command_reply(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<WhatsAppReplyRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let response = whatsapp_provider_runtime_service(&state)?
        .request_reply(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_command_forward(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(mut request): Json<WhatsAppForwardRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    if request
        .text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        request.text = whatsapp_forward_source_body(&state, &request.account_id, &message_id)
            .await?
            .or(whatsapp_forward_source_body(
                &state,
                &request.account_id,
                &request.from_provider_message_id,
            )
            .await?);
    }
    let response = whatsapp_provider_runtime_service(&state)?
        .request_forward(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

async fn whatsapp_forward_source_body(
    state: &AppState,
    account_id: &str,
    message_id_or_provider_id: &str,
) -> Result<Option<String>, ApiError> {
    if let Some(message) = message_store(state)?
        .message(message_id_or_provider_id)
        .await?
    {
        return Ok(Some(message.body_text));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?;
    let message = ProviderChannelMessageStore::new(pool.clone())
        .message_by_provider_record_id(account_id, message_id_or_provider_id, &["whatsapp_web"])
        .await
        .map_err(WhatsappWebError::from)?;
    Ok(message.map(|message| message.body_text))
}

pub(crate) async fn post_whatsapp_command_edit(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<WhatsAppEditRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let response = whatsapp_provider_runtime_service(&state)?
        .request_edit(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_command_delete(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<WhatsAppDeleteRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let response = whatsapp_provider_runtime_service(&state)?
        .request_delete(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_command_react(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<WhatsAppReactionRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let response = whatsapp_provider_runtime_service(&state)?
        .request_react(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}

pub(crate) async fn delete_whatsapp_command_react(
    State(state): State<AppState>,
    Path(_message_id): Path<String>,
    Json(request): Json<WhatsAppReactionRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let response = whatsapp_provider_runtime_service(&state)?
        .request_unreact(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    Ok(Json(response))
}
