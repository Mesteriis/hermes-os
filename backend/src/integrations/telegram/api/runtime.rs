use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use super::helpers::{AUDIT_ACTOR_ID, telegram_secret_store};
use crate::app::{ApiError, AppState};
use crate::domains::api_support::{api_audit_log, communication_ingestion_store};
use crate::integrations::telegram::runtime::{
    TelegramRuntimeRestartRequest, TelegramRuntimeStartContext, TelegramRuntimeStartRequest,
    TelegramRuntimeStatus, TelegramRuntimeStopRequest,
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
        state
            .telegram_runtime
            .status_for_account(
                &communication_ingestion_store(&state)?,
                &state.config,
                &query.account_id,
            )
            .await?,
    ))
}

pub(crate) async fn post_telegram_runtime_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStartRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let runtime = state.telegram_runtime.clone();
    let config = state.config.clone();
    let vault = state.vault.clone();
    let communication_store = communication_ingestion_store(&state)?;
    let secret_store = telegram_secret_store(&state)?;
    let context = TelegramRuntimeStartContext {
        communication_store: &communication_store,
        secret_store: &secret_store,
        secret_resolver: &vault,
        config: &config,
        event_bus: &state.event_bus,
        event_store_pool: state.database.pool().cloned(),
    };

    Ok(Json(runtime.start_account(&context, &request).await?))
}

pub(crate) async fn post_telegram_runtime_stop(
    State(state): State<AppState>,
    Json(request): Json<TelegramRuntimeStopRequest>,
) -> Result<Json<TelegramRuntimeStatus>, ApiError> {
    let status = state
        .telegram_runtime
        .stop_account_runtime(
            &communication_ingestion_store(&state)?,
            &state.config,
            &request,
        )
        .await?;

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
    let runtime = state.telegram_runtime.clone();
    let config = state.config.clone();
    let vault = state.vault.clone();
    let communication_store = communication_ingestion_store(&state)?;
    let secret_store = telegram_secret_store(&state)?;
    let context = TelegramRuntimeStartContext {
        communication_store: &communication_store,
        secret_store: &secret_store,
        secret_resolver: &vault,
        config: &config,
        event_bus: &state.event_bus,
        event_store_pool: state.database.pool().cloned(),
    };
    let status = runtime.restart_account_runtime(&context, &request).await?;

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
