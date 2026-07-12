use crate::integrations::telegram::client::{TelegramError, TelegramStore};

use super::super::actor::oldest_tdlib_message_id;
use super::super::commands::request_actor_history;
use super::super::models::{
    TelegramHistorySyncMode, TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::sync_history_tdlib::TdlibHistorySyncContext;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

impl TelegramRuntimeManager {
    pub(crate) async fn sync_history<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramHistorySyncRequest,
    ) -> Result<TelegramHistorySyncResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => {
                sync_fixture_history(context.telegram_store, &account.account_id, request).await
            }
            "tdlib_qr_authorized" => {
                let context = TdlibHistorySyncContext {
                    provider_account_store: context.provider_account_store,
                    provider_secret_binding_store: context.provider_secret_binding_store,
                    telegram_store: context.telegram_store,
                    secret_store: context.secret_store,
                    secret_resolver: context.secret_resolver,
                    config: context.config,
                    account: &account,
                    runtime_kind,
                    event_bridge: context.event_bridge.clone(),
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
            request.limit.unwrap_or(100),
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
