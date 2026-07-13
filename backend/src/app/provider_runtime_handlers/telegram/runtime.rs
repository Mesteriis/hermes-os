use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use super::helpers::AUDIT_ACTOR_ID;
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
use crate::application::telegram_runtime;
use crate::integrations::telegram::runtime::{
    TelegramRuntimeRestartRequest, TelegramRuntimeStartRequest, TelegramRuntimeStatus,
    TelegramRuntimeStopRequest,
};
use crate::platform::audit::NewApiAuditRecord;

#[derive(Deserialize)]
pub(crate) struct TelegramRuntimeStatusQuery {
    pub(crate) account_id: String,
}

pub(crate) async fn get_telegram_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<TelegramRuntimeStatusQuery>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    Ok(Json(
        telegram_runtime::runtime_status(&runtime_context, &query.account_id).await?,
    ))
}

pub(crate) async fn post_telegram_runtime_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStartRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    Ok(Json(
        telegram_runtime::start_runtime(&runtime_context, &request).await?,
    ))
}

pub(crate) async fn post_telegram_runtime_stop(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStopRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let status = telegram_runtime::stop_runtime(&runtime_context, &request).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_runtime_stop(
            AUDIT_ACTOR_ID,
            &status.account_id,
            &status.provider_kind,
            &status.runtime_kind,
            &status.status,
        ))
        .await?;

    Ok(Json(status))
}

pub(crate) async fn post_telegram_runtime_restart(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeRestartRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let status = telegram_runtime::restart_runtime(&runtime_context, &request).await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_runtime_restart(
            AUDIT_ACTOR_ID,
            &status.account_id,
            &status.provider_kind,
            &status.runtime_kind,
            &status.status,
        ))
        .await?;

    Ok(Json(status))
}
