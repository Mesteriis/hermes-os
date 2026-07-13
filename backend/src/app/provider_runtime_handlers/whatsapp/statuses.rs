//! Hidden WebView WhatsApp status publishing endpoint.

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::json;

use crate::app::api_support::stores::integration_stores::{
    whatsapp_provider_runtime_service, whatsapp_secret_reference_store,
};
use crate::app::{ApiError, AppState};
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppProviderCommandResponse, WhatsAppStatusPublishRequest,
};

use super::{
    capture_whatsapp_status_publish_runtime_signal, publish_whatsapp_command_event, required_string,
};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppStatusPublishApiRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) idempotency_key: String,
    pub(crate) account_id: String,
    pub(crate) text: String,
}

pub(crate) async fn post_whatsapp_status_publish(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppStatusPublishApiRequest>,
) -> Result<Json<WhatsAppProviderCommandResponse>, ApiError> {
    let request = validate_whatsapp_status_publish_request(request)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .request_publish_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    publish_whatsapp_command_event(&state, &response).await?;
    capture_whatsapp_status_publish_runtime_signal(
        &state,
        &response.account_id,
        &response.command_id,
        "requested",
        json!({
            "command_id": response.command_id,
            "command_kind": response.command_kind,
            "provider_chat_id": response.provider_chat_id,
            "status": response.status,
        }),
    )
    .await?;
    if response.status == "blocked" {
        capture_whatsapp_status_publish_runtime_signal(
            &state,
            &response.account_id,
            &response.command_id,
            "failed",
            json!({
                "command_id": response.command_id,
                "command_kind": response.command_kind,
                "provider_chat_id": response.provider_chat_id,
                "status": "failed",
                "error": response
                    .runtime_blockers
                    .first()
                    .cloned()
                    .or_else(|| response.last_error.clone()),
                "runtime_blockers": response.runtime_blockers,
            }),
        )
        .await?;
    }
    Ok(Json(response))
}

fn validate_whatsapp_status_publish_request(
    request: WhatsAppStatusPublishApiRequest,
) -> Result<WhatsAppStatusPublishRequest, ApiError> {
    Ok(WhatsAppStatusPublishRequest {
        command_id: request.command_id,
        idempotency_key: required_string("idempotency_key", &request.idempotency_key)?,
        account_id: required_string("account_id", &request.account_id)?,
        text: required_string("text", &request.text)?,
    })
}
