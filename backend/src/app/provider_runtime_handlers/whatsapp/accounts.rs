//! Hidden WebView WhatsApp account lifecycle and setup endpoints.

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;

use crate::app::api_support::{
    messaging_integrations::*,
    stores::{domain_stores::*, integration_stores::*},
    whatsapp_capabilities::*,
};
use crate::app::error::types::ApiError;
use crate::app::signal_hub_support::{
    provider_account_or_not_found, remove_provider_account_signal_connection,
    sync_provider_account_signal_connection, sync_whatsapp_runtime_signal_connection,
};
use crate::app::state::AppState;
use crate::integrations::whatsapp::client::models::{
    WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupResponse,
};
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppPairCodeSession, WhatsAppPairCodeStartRequest, WhatsAppProviderRuntimeShape,
    WhatsAppQrLinkSession, WhatsAppQrLinkStartRequest, WhatsAppRuntimeHealth,
    WhatsAppRuntimeRelinkRequest, WhatsAppRuntimeRemoveRequest, WhatsAppRuntimeRemoveResponse,
    WhatsAppRuntimeRevokeRequest, WhatsAppRuntimeStartRequest, WhatsAppRuntimeStatus,
    WhatsAppRuntimeStopRequest,
};

use super::{
    capture_whatsapp_runtime_lifecycle_signal, publish_whatsapp_runtime_status_event,
    publish_whatsapp_session_link_state_event,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsappAccountSummary {
    pub(crate) account_id: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_shape: Option<String>,
    pub(crate) display_name: String,
    pub(crate) external_account_id: String,
    pub(crate) runtime: Option<String>,
    pub(crate) lifecycle_state: Option<String>,
    pub(crate) created_at: chrono::DateTime<Utc>,
    pub(crate) updated_at: chrono::DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsappAccountListResponse {
    pub(crate) items: Vec<WhatsappAccountSummary>,
}

#[derive(Deserialize)]
pub(crate) struct WhatsappRuntimeAccountQuery {
    pub(crate) account_id: String,
}

#[derive(Deserialize)]
pub(crate) struct WhatsappAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

pub(crate) async fn get_whatsapp_capabilities(
    State(_state): State<AppState>,
) -> Result<Json<WhatsappCapabilitiesResponse>, ApiError> {
    Ok(Json(WhatsappCapabilitiesResponse::current(
        WhatsAppProviderRuntimeShape::WebCompanion,
    )))
}

pub(crate) async fn get_whatsapp_account_capabilities(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<WhatsappCapabilitiesResponse>, ApiError> {
    let status = whatsapp_provider_runtime_service(&state)?
        .runtime_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &account_id,
        )
        .await?;
    Ok(Json(WhatsappCapabilitiesResponse::current_for_account(
        &status,
    )))
}

pub(crate) async fn get_whatsapp_accounts(
    State(state): State<AppState>,
    Query(query): Query<WhatsappAccountsQuery>,
) -> Result<Json<WhatsappAccountListResponse>, ApiError> {
    let items = communication_provider_account_store(&state)?
        .list()
        .await?
        .into_iter()
        .filter(|account| account.provider_kind.is_whatsapp())
        .filter(|account| {
            query.include_removed
                || account
                    .config
                    .get("lifecycle_state")
                    .and_then(Value::as_str)
                    != Some("removed")
        })
        .map(|account| WhatsappAccountSummary {
            provider_shape: account
                .config
                .get("provider_shape")
                .and_then(Value::as_str)
                .map(str::to_owned),
            runtime: account
                .config
                .get("runtime")
                .and_then(Value::as_str)
                .map(str::to_owned),
            lifecycle_state: account
                .config
                .get("lifecycle_state")
                .and_then(Value::as_str)
                .map(str::to_owned),
            account_id: account.account_id,
            provider_kind: account.provider_kind.as_str().to_owned(),
            display_name: account.display_name,
            external_account_id: account.external_account_id,
            created_at: account.created_at,
            updated_at: account.updated_at,
        })
        .collect();

    Ok(Json(WhatsappAccountListResponse { items }))
}

pub(crate) async fn get_whatsapp_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<WhatsappRuntimeAccountQuery>,
) -> Result<Json<WhatsAppRuntimeStatus>, ApiError> {
    Ok(Json(
        whatsapp_provider_runtime_service(&state)?
            .runtime_status(
                &whatsapp_secret_reference_store(&state)?,
                &state.vault,
                &query.account_id,
            )
            .await?,
    ))
}

pub(crate) async fn post_whatsapp_runtime_start(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeStartRequest>,
) -> Result<Json<WhatsAppRuntimeStatus>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .start_runtime(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, "runtime_start").await?;
    publish_whatsapp_runtime_status_event(&state, &status, "runtime_start").await?;
    Ok(Json(status))
}

pub(crate) async fn post_whatsapp_runtime_stop(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeStopRequest>,
) -> Result<Json<WhatsAppRuntimeStatus>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .stop_runtime(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, "runtime_stop").await?;
    publish_whatsapp_runtime_status_event(&state, &status, "runtime_stop").await?;
    Ok(Json(status))
}

pub(crate) async fn post_whatsapp_runtime_revoke(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeRevokeRequest>,
) -> Result<Json<WhatsAppRuntimeStatus>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .revoke_runtime(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, "runtime_revoke").await?;
    publish_whatsapp_runtime_status_event(&state, &status, "runtime_revoke").await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        "runtime_revoke",
        status.updated_at,
    )
    .await?;
    Ok(Json(status))
}

pub(crate) async fn post_whatsapp_runtime_relink(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeRelinkRequest>,
) -> Result<Json<WhatsAppRuntimeStatus>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .relink_runtime(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, "runtime_relink").await?;
    publish_whatsapp_runtime_status_event(&state, &status, "runtime_relink").await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        "runtime_relink",
        status.updated_at,
    )
    .await?;
    Ok(Json(status))
}

pub(crate) async fn post_whatsapp_runtime_rotate(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeRelinkRequest>,
) -> Result<Json<WhatsAppRuntimeStatus>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .relink_runtime(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, "runtime_rotate").await?;
    publish_whatsapp_runtime_status_event(&state, &status, "runtime_rotate").await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        "runtime_rotate",
        status.updated_at,
    )
    .await?;
    Ok(Json(status))
}

pub(crate) async fn post_whatsapp_runtime_remove(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeRemoveRequest>,
) -> Result<Json<WhatsAppRuntimeRemoveResponse>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let response = whatsapp_provider_runtime_service(&state)?
        .remove_runtime(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    remove_provider_account_signal_connection(&state, &provider_account).await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .runtime_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request.account_id,
        )
        .await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, "runtime_remove").await?;
    publish_whatsapp_runtime_status_event(&state, &status, "runtime_remove").await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        "runtime_remove",
        status.updated_at,
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn get_whatsapp_runtime_health(
    State(state): State<AppState>,
    Query(query): Query<WhatsappRuntimeAccountQuery>,
) -> Result<Json<WhatsAppRuntimeHealth>, ApiError> {
    Ok(Json(
        whatsapp_provider_runtime_service(&state)?
            .runtime_health(
                &whatsapp_secret_reference_store(&state)?,
                &state.vault,
                &query.account_id,
            )
            .await?,
    ))
}

pub(crate) async fn post_whatsapp_qr_link_start(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppQrLinkStartRequest>,
) -> Result<Json<WhatsAppQrLinkSession>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let session = whatsapp_provider_runtime_service(&state)?
        .start_qr_link(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .runtime_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request.account_id,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, "login_qr_start").await?;
    publish_whatsapp_runtime_status_event(&state, &status, "login_qr_start").await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        "login_qr_start",
        status.updated_at,
    )
    .await?;
    Ok(Json(session))
}

pub(crate) async fn post_whatsapp_pair_code_link_start(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppPairCodeStartRequest>,
) -> Result<Json<WhatsAppPairCodeSession>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let session = whatsapp_provider_runtime_service(&state)?
        .start_pair_code_link(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .runtime_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request.account_id,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, "login_pair_code_start").await?;
    publish_whatsapp_runtime_status_event(&state, &status, "login_pair_code_start").await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        "login_pair_code_start",
        status.updated_at,
    )
    .await?;
    Ok(Json(session))
}

pub(crate) async fn post_whatsapp_account(
    State(state): State<AppState>,
    Json(request): Json<WhatsappLiveAccountSetupRequest>,
) -> Result<Json<WhatsappWebAccountSetupResponse>, ApiError> {
    let response = whatsapp_provider_runtime_service(&state)?
        .setup_live_blocked_account(&request)
        .await?;
    let account = provider_account_or_not_found(&state, &response.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, None).await?;
    Ok(Json(response))
}

pub(crate) async fn get_whatsapp_sessions(
    State(state): State<AppState>,
    Query(query): Query<WhatsappWebListQuery>,
) -> Result<Json<WhatsappWebSessionListResponse>, ApiError> {
    let items = whatsapp_provider_runtime_service(&state)?
        .list_sessions(query.account_id.as_deref(), query.limit.unwrap_or(50))
        .await?;

    Ok(Json(WhatsappWebSessionListResponse { items }))
}
