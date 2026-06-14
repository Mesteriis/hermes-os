use crate::domains::mail::core::CommunicationIngestionStore;
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::super::commands::request_actor_chats;
use super::super::models::{TelegramChatSyncRequest, TelegramChatSyncResponse};
use super::super::status::account_runtime_kind;
use super::TelegramRuntimeManager;
use super::account::load_active_account;

impl TelegramRuntimeManager {
    pub async fn sync_chats(
        &self,
        communication_store: &CommunicationIngestionStore,
        telegram_store: &TelegramStore,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        request: &TelegramChatSyncRequest,
    ) -> Result<TelegramChatSyncResponse, TelegramError> {
        request.validate()?;
        let account = load_active_account(communication_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => {
                let items = telegram_store
                    .list_chats(Some(&account.account_id), request.limit.unwrap_or(50))
                    .await?;
                Ok(TelegramChatSyncResponse {
                    account_id: account.account_id,
                    runtime_kind,
                    status: "synced".to_owned(),
                    synced_count: items.len(),
                    items,
                })
            }
            "tdlib_qr_authorized" => {
                let command_tx = self
                    .ensure_tdlib_actor(
                        communication_store,
                        secret_store,
                        secret_resolver,
                        config,
                        &account,
                    )
                    .await?;
                let snapshots =
                    request_actor_chats(command_tx, request.limit.unwrap_or(50) as i32).await?;
                for snapshot in &snapshots {
                    telegram_store
                        .ingest_tdlib_chat_snapshot(&account.account_id, snapshot)
                        .await?;
                }
                let items = telegram_store
                    .list_chats(Some(&account.account_id), request.limit.unwrap_or(50))
                    .await?;
                Ok(TelegramChatSyncResponse {
                    account_id: account.account_id,
                    runtime_kind,
                    status: "synced".to_owned(),
                    synced_count: snapshots.len(),
                    items,
                })
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
