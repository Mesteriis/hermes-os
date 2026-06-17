use std::collections::BTreeSet;

use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;

use super::super::commands::{request_actor_chat_folders, request_actor_chats};
use super::super::models::{TelegramChatSyncRequest, TelegramChatSyncResponse};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

impl TelegramRuntimeManager {
    pub(crate) async fn sync_chats<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramChatSyncRequest,
    ) -> Result<TelegramChatSyncResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account = load_active_account(context.communication_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => {
                let items = context
                    .telegram_store
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
                        context.communication_store,
                        context.secret_store,
                        context.secret_resolver,
                        context.config,
                        &account,
                        context.event_bridge.clone(),
                    )
                    .await?;
                let snapshots =
                    request_actor_chats(command_tx.clone(), request.limit.unwrap_or(50) as i32)
                        .await?;
                for snapshot in &snapshots {
                    context
                        .telegram_store
                        .ingest_tdlib_chat_snapshot(&account.account_id, snapshot)
                        .await?;
                }
                let folder_ids = tdlib_folder_ids_from_chat_snapshots(&snapshots);
                if !folder_ids.is_empty() {
                    let folder_snapshots =
                        request_actor_chat_folders(command_tx, folder_ids).await?;
                    context
                        .telegram_store
                        .apply_provider_chat_folders(&account.account_id, &folder_snapshots)
                        .await?;
                }
                let items = context
                    .telegram_store
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

fn tdlib_folder_ids_from_chat_snapshots(
    snapshots: &[crate::integrations::telegram::tdjson::TelegramTdlibChatSnapshot],
) -> Vec<i64> {
    let mut folder_ids = BTreeSet::new();
    for snapshot in snapshots {
        let Some(lists) = snapshot.raw.get("positions").and_then(Value::as_array) else {
            continue;
        };
        for list in lists {
            let Some(chat_list) = list.get("list").and_then(Value::as_object) else {
                continue;
            };
            if chat_list.get("@type").and_then(Value::as_str) != Some("chatListFolder") {
                continue;
            }
            if let Some(folder_id) = chat_list.get("chat_folder_id").and_then(Value::as_i64) {
                folder_ids.insert(folder_id);
            }
        }
    }
    folder_ids.into_iter().collect()
}
