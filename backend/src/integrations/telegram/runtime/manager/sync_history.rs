use crate::domains::mail::core::CommunicationIngestionStore;
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::super::actor::oldest_tdlib_message_id;
use super::super::commands::request_actor_history;
use super::super::models::{
    TelegramHistorySyncMode, TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};
use super::super::status::account_runtime_kind;
use super::TelegramRuntimeManager;
use super::account::load_active_account;
use super::sync_history_tdlib::TdlibHistorySyncContext;

impl TelegramRuntimeManager {
    pub async fn sync_history(
        &self,
        communication_store: &CommunicationIngestionStore,
        telegram_store: &TelegramStore,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        request: &TelegramHistorySyncRequest,
    ) -> Result<TelegramHistorySyncResponse, TelegramError> {
        request.validate()?;
        let account = load_active_account(communication_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => sync_fixture_history(telegram_store, &account.account_id, request).await,
            "tdlib_qr_authorized" => {
                let context = TdlibHistorySyncContext {
                    communication_store,
                    telegram_store,
                    secret_store,
                    secret_resolver,
                    config,
                    account: &account,
                    runtime_kind,
                };
                self.sync_tdlib_history(context, request).await
            }
            "live_blocked" => Err(TelegramError::InvalidRequest(
                "account runtime is blocked until live TDLib is enabled".to_owned(),
            )),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram runtime `{other}`"
            ))),
        }
    }
}

async fn sync_fixture_history(
    telegram_store: &TelegramStore,
    account_id: &str,
    request: &TelegramHistorySyncRequest,
) -> Result<TelegramHistorySyncResponse, TelegramError> {
    let items = telegram_store
        .recent_messages(
            Some(account_id),
            Some(&request.provider_chat_id),
            request.limit.unwrap_or(50),
        )
        .await?;
    Ok(TelegramHistorySyncResponse {
        account_id: account_id.to_owned(),
        provider_chat_id: request.provider_chat_id.trim().to_owned(),
        runtime_kind: "fixture".to_owned(),
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: false,
        next_from_message_id: None,
        items,
    })
}
