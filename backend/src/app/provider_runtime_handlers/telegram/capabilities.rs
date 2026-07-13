use axum::Json;
use axum::extract::{Path, State};

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
use crate::app::{ApiError, AppState};

pub(crate) async fn get_telegram_capabilities(
    State(state): State<AppState>,
) -> Result<Json<TelegramCapabilitiesResponse>, ApiError> {
    Ok(Json(TelegramCapabilitiesResponse::current(&state.config)))
}

pub(crate) async fn get_telegram_account_capabilities(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<TelegramCapabilitiesResponse>, ApiError> {
    let account = telegram_provider_runtime_service(&state)?
        .telegram_account_record(&account_id)
        .await?;
    Ok(Json(TelegramCapabilitiesResponse::current_for_account(
        &state.config,
        &account,
    )))
}
