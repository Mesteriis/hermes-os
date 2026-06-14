use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use super::helpers::telegram_secret_store;
use crate::app::{ApiError, AppState};
use crate::domains::api_support::communication_ingestion_store;
use crate::integrations::telegram::runtime::{TelegramRuntimeStartRequest, TelegramRuntimeStatus};

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

    Ok(Json(
        runtime
            .start_account(
                &communication_store,
                &secret_store,
                &vault,
                &config,
                &request,
            )
            .await?,
    ))
}
