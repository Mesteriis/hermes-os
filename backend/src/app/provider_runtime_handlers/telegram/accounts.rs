use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use super::helpers::{AUDIT_ACTOR_ID, telegram_api_hash_from_config, telegram_secret_store};
use crate::app::api_support::{api_audit_log, ensure_fixture_routes_enabled, telegram_store};
use crate::app::{ApiError, AppState};
use crate::integrations::telegram::client::{
    TelegramAccountLifecycleResponse, TelegramAccountListResponse, TelegramAccountSetupRequest,
    TelegramAccountSetupResponse, TelegramLiveAccountSetupRequest, TelegramSecretVault,
};
use crate::platform::audit::NewApiAuditRecord;

pub(crate) async fn post_telegram_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<TelegramAccountSetupRequest>,
) -> Result<Json<TelegramAccountSetupResponse>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    Ok(Json(
        telegram_store(&state)?
            .setup_fixture_account(&request)
            .await?,
    ))
}

pub(crate) async fn post_telegram_account(
    State(state): State<AppState>,
    Json(request): Json<TelegramLiveAccountSetupRequest>,
) -> Result<Json<TelegramAccountSetupResponse>, ApiError> {
    let request = request
        .with_inferred_qr_authorization()
        .with_app_credentials(
            state.config.telegram_api_id(),
            telegram_api_hash_from_config(&state.config),
        );

    Ok(Json(
        telegram_store(&state)?
            .setup_live_blocked_account(
                &telegram_secret_store(&state)?,
                &TelegramSecretVault::host(state.vault.clone()),
                &request,
            )
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct TelegramAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

pub(crate) async fn get_telegram_accounts(
    State(state): State<AppState>,
    Query(query): Query<TelegramAccountsQuery>,
) -> Result<Json<TelegramAccountListResponse>, ApiError> {
    let items = telegram_store(&state)?
        .list_accounts(query.include_removed)
        .await?;

    Ok(Json(TelegramAccountListResponse { items }))
}

pub(crate) async fn post_telegram_account_logout(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<TelegramAccountLifecycleResponse>, ApiError> {
    let account = telegram_store(&state)?.logout_account(&account_id).await?;
    let stopped_runtime_actor = state.telegram_runtime.stop_account(&account.account_id)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_account_logout(
            AUDIT_ACTOR_ID,
            &account.account_id,
            &account.provider_kind,
            &account.lifecycle_state,
        ))
        .await?;

    Ok(Json(TelegramAccountLifecycleResponse {
        account,
        stopped_runtime_actor,
    }))
}

pub(crate) async fn delete_telegram_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<TelegramAccountLifecycleResponse>, ApiError> {
    let account = telegram_store(&state)?.remove_account(&account_id).await?;
    let stopped_runtime_actor = state.telegram_runtime.stop_account(&account.account_id)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_account_remove(
            AUDIT_ACTOR_ID,
            &account.account_id,
            &account.provider_kind,
            &account.lifecycle_state,
        ))
        .await?;

    Ok(Json(TelegramAccountLifecycleResponse {
        account,
        stopped_runtime_actor,
    }))
}
