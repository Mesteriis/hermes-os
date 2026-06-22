use axum::Json;
use axum::extract::{Query, State};

use crate::app::api_support::{
    WhatsappCapabilitiesResponse, WhatsappWebListQuery, WhatsappWebMessageListResponse,
    WhatsappWebSessionListResponse, ensure_fixture_routes_enabled, whatsapp_fixture_ingest_service,
    whatsapp_provider_runtime_service,
};
use crate::app::{ApiError, AppState};
use crate::application::provider_runtime_contracts::{
    NewWhatsappWebMessage, WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse,
    WhatsappWebMessageIngestResult,
};

pub(crate) async fn get_whatsapp_capabilities(
    State(_state): State<AppState>,
) -> Result<Json<WhatsappCapabilitiesResponse>, ApiError> {
    Ok(Json(WhatsappCapabilitiesResponse::current()))
}

pub(crate) async fn post_whatsapp_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<WhatsappWebAccountSetupRequest>,
) -> Result<Json<WhatsappWebAccountSetupResponse>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    Ok(Json(
        whatsapp_provider_runtime_service(&state)?
            .setup_fixture_account(&request)
            .await?,
    ))
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

pub(crate) async fn post_whatsapp_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessage>,
) -> Result<Json<WhatsappWebMessageIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    Ok(Json(
        whatsapp_fixture_ingest_service(&state)?
            .ingest_message(&request)
            .await?,
    ))
}

pub(crate) async fn get_whatsapp_messages(
    State(state): State<AppState>,
    Query(query): Query<WhatsappWebListQuery>,
) -> Result<Json<WhatsappWebMessageListResponse>, ApiError> {
    let items = whatsapp_provider_runtime_service(&state)?
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(WhatsappWebMessageListResponse { items }))
}
