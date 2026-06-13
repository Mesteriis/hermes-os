use chrono::Utc;
use serde_json::Value;

use crate::domains::mail::core::{
    CommunicationIngestionStore, CommunicationProviderKind, ProviderAccount,
};
use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson;
use crate::platform::config::AppConfig;

use super::models::TelegramRuntimeStatus;
use super::state::{TelegramRuntimeActorState, TelegramRuntimeState};
use super::validation::validate_non_empty;

pub(super) async fn load_telegram_account(
    communication_store: &CommunicationIngestionStore,
    account_id: &str,
) -> Result<ProviderAccount, TelegramError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let account = communication_store
        .provider_account(&account_id)
        .await?
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram account `{account_id}` is not configured"
            ))
        })?;

    if !account.provider_kind.is_telegram() {
        return Err(TelegramError::InvalidRequest(format!(
            "account `{}` is not a Telegram provider account",
            account.account_id
        )));
    }

    Ok(account)
}

pub(super) fn status_from_account(
    config: &AppConfig,
    account: &ProviderAccount,
    actor_state: Option<TelegramRuntimeActorState>,
) -> TelegramRuntimeStatus {
    let runtime_kind = account_runtime_kind(account);
    let default_state = default_state_for_runtime(&runtime_kind);
    let actor_state = actor_state.unwrap_or(default_state);
    let telegram_app_credentials_configured =
        config.telegram_api_id().is_some() && config.telegram_api_hash().is_some();
    let tdjson_runtime_available = tdjson::runtime_available(config.tdjson_path());
    let live_send_available = runtime_kind == "tdlib_qr_authorized"
        && actor_state.status == TelegramRuntimeState::Running
        && tdjson_runtime_available
        && telegram_app_credentials_configured;

    TelegramRuntimeStatus {
        account_id: account.account_id.clone(),
        provider_kind: account.provider_kind.as_str().to_owned(),
        runtime_kind: runtime_kind.clone(),
        status: actor_state.status.as_str().to_owned(),
        fixture_runtime: runtime_kind == "fixture",
        tdjson_runtime_available,
        telegram_app_credentials_configured,
        live_send_available,
        last_error: actor_state.last_error,
        updated_at: actor_state.updated_at,
    }
}

fn default_state_for_runtime(runtime_kind: &str) -> TelegramRuntimeActorState {
    let now = Utc::now();
    match runtime_kind {
        "live_blocked" => TelegramRuntimeActorState {
            status: TelegramRuntimeState::Blocked,
            last_error: Some("account runtime is blocked until live TDLib is enabled".to_owned()),
            updated_at: now,
        },
        _ => TelegramRuntimeActorState {
            status: TelegramRuntimeState::Stopped,
            last_error: None,
            updated_at: now,
        },
    }
}

pub(super) fn account_runtime_kind(account: &ProviderAccount) -> String {
    account
        .config
        .get("runtime")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(match account.provider_kind {
            CommunicationProviderKind::TelegramUser | CommunicationProviderKind::TelegramBot => {
                "unknown"
            }
            _ => "unsupported",
        })
        .to_owned()
}
