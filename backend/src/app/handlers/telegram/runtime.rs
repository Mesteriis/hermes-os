use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use super::helpers::AUDIT_ACTOR_ID;
use crate::app::api_support::api_audit_log;
use crate::app::{ApiError, AppState, telegram_application};
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
    Ok(Json(
        telegram_application::runtime_status(&state, &query.account_id).await?,
    ))
}

pub(crate) async fn post_telegram_runtime_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStartRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    Ok(Json(
        telegram_application::start_runtime(&state, &request).await?,
    ))
}

pub(crate) async fn post_telegram_runtime_stop(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStopRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let status = telegram_application::stop_runtime(&state, &request).await?;

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
    let status = telegram_application::restart_runtime(&state, &request).await?;

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
