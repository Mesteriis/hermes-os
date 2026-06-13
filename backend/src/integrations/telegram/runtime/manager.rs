use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use chrono::Utc;

use crate::domains::mail::background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::domains::mail::core::{CommunicationIngestionStore, ProviderAccount};
use crate::domains::mail::storage::MailStorageStore;
use crate::integrations::telegram::client::{
    TelegramError, TelegramManualSendRequest, TelegramManualSendResponse, TelegramStore,
    ensure_telegram_account_active, telegram_text_preview_hash,
};
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::actor::{oldest_tdlib_message_id, optional_telegram_session_key, spawn_tdlib_actor};
use super::commands::{
    request_actor_chats, request_actor_download_file, request_actor_history, request_actor_send,
};
use super::media::persist_downloaded_media;
use super::models::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncMode,
    TelegramHistorySyncRequest, TelegramHistorySyncResponse, TelegramMediaDownloadRequest,
    TelegramMediaDownloadResponse, TelegramRuntimeStartRequest, TelegramRuntimeStatus,
};
use super::state::{
    TelegramRuntimeActorHandle, TelegramRuntimeActorState, TelegramRuntimeCommand,
    TelegramRuntimeState,
};
use super::status::{account_runtime_kind, load_telegram_account, status_from_account};
use super::validation::validate_non_empty;

#[derive(Clone, Default)]
pub struct TelegramRuntimeManager {
    actors: Arc<Mutex<HashMap<String, TelegramRuntimeActorHandle>>>,
}

pub(crate) struct TelegramMediaDownloadContext<'a, S: SecretResolver + Sync + ?Sized> {
    pub(crate) communication_store: &'a CommunicationIngestionStore,
    pub(crate) telegram_store: &'a TelegramStore,
    pub(crate) mail_store: &'a MailStorageStore,
    pub(crate) secret_store: &'a SecretReferenceStore,
    pub(crate) secret_resolver: &'a S,
    pub(crate) config: &'a AppConfig,
}

impl TelegramRuntimeManager {
    pub async fn status_for_account(
        &self,
        communication_store: &CommunicationIngestionStore,
        config: &AppConfig,
        account_id: &str,
    ) -> Result<TelegramRuntimeStatus, TelegramError> {
        let account = load_telegram_account(communication_store, account_id).await?;
        let actor_state = self.actor_state(&account.account_id)?;

        Ok(status_from_account(config, &account, actor_state))
    }

    pub async fn start_account(
        &self,
        communication_store: &CommunicationIngestionStore,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        request: &TelegramRuntimeStartRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError> {
        request.validate()?;
        let account = load_telegram_account(communication_store, &request.account_id).await?;
        ensure_telegram_account_active(&account)?;
        let session_encryption_key = optional_telegram_session_key(
            communication_store,
            secret_store,
            secret_resolver,
            &account.account_id,
        )
        .await?;
        let runtime_kind = account_runtime_kind(&account);
        let now = Utc::now();
        let (actor_state, command_tx) = match runtime_kind.as_str() {
            "fixture" => TelegramRuntimeActorState {
                status: TelegramRuntimeState::Running,
                last_error: None,
                updated_at: now,
            }
            .without_command(),
            "tdlib_qr_authorized" => {
                match spawn_tdlib_actor(config.clone(), account.clone(), session_encryption_key) {
                    Ok(command_tx) => TelegramRuntimeActorState {
                        status: TelegramRuntimeState::Running,
                        last_error: None,
                        updated_at: now,
                    }
                    .with_command(command_tx),
                    Err(error) => TelegramRuntimeActorState {
                        status: TelegramRuntimeState::Degraded,
                        last_error: Some(error.to_string()),
                        updated_at: now,
                    }
                    .without_command(),
                }
            }
            "live_blocked" => TelegramRuntimeActorState {
                status: TelegramRuntimeState::Blocked,
                last_error: Some(
                    "account runtime is blocked until live TDLib is enabled".to_owned(),
                ),
                updated_at: now,
            }
            .without_command(),
            other => TelegramRuntimeActorState {
                status: TelegramRuntimeState::Error,
                last_error: Some(format!("unsupported Telegram runtime `{other}`")),
                updated_at: now,
            }
            .without_command(),
        };

        self.set_actor_handle(
            account.account_id.clone(),
            TelegramRuntimeActorHandle {
                state: actor_state.clone(),
                command_tx,
            },
        )?;

        Ok(status_from_account(config, &account, Some(actor_state)))
    }

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
        let account = load_telegram_account(communication_store, &request.account_id).await?;
        ensure_telegram_account_active(&account)?;
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
        let account = load_telegram_account(communication_store, &request.account_id).await?;
        ensure_telegram_account_active(&account)?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => {
                let items = telegram_store
                    .recent_messages(
                        Some(&account.account_id),
                        Some(&request.provider_chat_id),
                        request.limit.unwrap_or(50),
                    )
                    .await?;
                Ok(TelegramHistorySyncResponse {
                    account_id: account.account_id,
                    provider_chat_id: request.provider_chat_id.trim().to_owned(),
                    runtime_kind,
                    status: "synced".to_owned(),
                    synced_count: items.len(),
                    has_more: false,
                    next_from_message_id: None,
                    items,
                })
            }
            "tdlib_qr_authorized" => {
                let mode = request.mode();
                if mode == TelegramHistorySyncMode::Full {
                    let chat = telegram_store
                        .telegram_chat(&account.account_id, &request.provider_chat_id)
                        .await?
                        .ok_or_else(|| {
                            TelegramError::InvalidRequest(format!(
                                "Telegram chat `{}` is not synced for account `{}`",
                                request.provider_chat_id.trim(),
                                account.account_id
                            ))
                        })?;
                    if chat.chat_kind != "private" {
                        return Err(TelegramError::InvalidRequest(
                            "full Telegram history sync is only allowed for private chats; group and channel history must be paged with mode=older"
                                .to_owned(),
                        ));
                    }
                }
                let command_tx = self
                    .ensure_tdlib_actor(
                        communication_store,
                        secret_store,
                        secret_resolver,
                        config,
                        &account,
                    )
                    .await?;
                let snapshots = request_actor_history(
                    command_tx,
                    request.provider_chat_id.trim().to_owned(),
                    request.from_message_id,
                    request.limit.unwrap_or(50) as i32,
                    mode,
                )
                .await?;
                let next_from_message_id = oldest_tdlib_message_id(&snapshots);
                let has_more = mode != TelegramHistorySyncMode::Full
                    && next_from_message_id.is_some()
                    && snapshots.len() >= request.limit.unwrap_or(50) as usize;
                let import_batch_id = format!(
                    "telegram-tdlib-history-sync:{}:{}",
                    account.account_id,
                    request.provider_chat_id.trim()
                );
                for snapshot in &snapshots {
                    telegram_store
                        .ingest_tdlib_message_snapshot(
                            &account.account_id,
                            snapshot,
                            &import_batch_id,
                        )
                        .await?;
                }
                let items = telegram_store
                    .recent_messages(
                        Some(&account.account_id),
                        Some(&request.provider_chat_id),
                        request.limit.unwrap_or(50),
                    )
                    .await?;
                Ok(TelegramHistorySyncResponse {
                    account_id: account.account_id,
                    provider_chat_id: request.provider_chat_id.trim().to_owned(),
                    runtime_kind,
                    status: "synced".to_owned(),
                    synced_count: snapshots.len(),
                    has_more,
                    next_from_message_id,
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

    pub async fn send_manual_message(
        &self,
        communication_store: &CommunicationIngestionStore,
        telegram_store: &TelegramStore,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError> {
        request.validate()?;
        let account = load_telegram_account(communication_store, &request.account_id).await?;
        ensure_telegram_account_active(&account)?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => telegram_store.manual_send_message(request).await,
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
                let snapshot = request_actor_send(command_tx, request.clone()).await?;
                let import_batch_id = format!(
                    "telegram-manual-send:{}:{}",
                    account.account_id,
                    request.command_id.trim()
                );
                let result = telegram_store
                    .ingest_tdlib_message_snapshot(&account.account_id, &snapshot, &import_batch_id)
                    .await?;
                Ok(TelegramManualSendResponse {
                    raw_record_id: result.raw_record_id,
                    message_id: result.message_id,
                    account_id: account.account_id,
                    provider_chat_id: request.provider_chat_id.trim().to_owned(),
                    delivery_state: snapshot.delivery_state.as_str().to_owned(),
                    status: "sent".to_owned(),
                    runtime_kind,
                    rendered_preview_hash: telegram_text_preview_hash(&request.text),
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

    pub(crate) async fn download_media<S: SecretResolver + Sync + ?Sized>(
        &self,
        context: TelegramMediaDownloadContext<'_, S>,
        request: &TelegramMediaDownloadRequest,
    ) -> Result<TelegramMediaDownloadResponse, TelegramError> {
        request.validate()?;
        let account =
            load_telegram_account(context.communication_store, &request.account_id).await?;
        ensure_telegram_account_active(&account)?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => Err(TelegramError::InvalidRequest(
                "Telegram media downloads require an enabled TDLib actor".to_owned(),
            )),
            "tdlib_qr_authorized" => {
                let command_tx = self
                    .ensure_tdlib_actor(
                        context.communication_store,
                        context.secret_store,
                        context.secret_resolver,
                        context.config,
                        &account,
                    )
                    .await?;
                let file = request_actor_download_file(
                    command_tx,
                    request.tdlib_file_id,
                    request.priority.unwrap_or(16),
                )
                .await?;
                let mut response = persist_downloaded_media(
                    context.telegram_store,
                    context.mail_store,
                    request,
                    &file,
                    Path::new(DEFAULT_MAIL_SYNC_BLOB_ROOT),
                )
                .await?;
                response.runtime_kind = runtime_kind;
                Ok(response)
            }
            "live_blocked" => Err(TelegramError::InvalidRequest(
                "account runtime is blocked until live TDLib is enabled".to_owned(),
            )),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram runtime `{other}`"
            ))),
        }
    }

    fn actor_state(
        &self,
        account_id: &str,
    ) -> Result<Option<TelegramRuntimeActorState>, TelegramError> {
        let actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors.get(account_id).map(|handle| handle.state.clone()))
    }

    pub fn stop_account(&self, account_id: &str) -> Result<bool, TelegramError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let mut actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors.remove(&account_id).is_some())
    }

    fn set_actor_handle(
        &self,
        account_id: String,
        actor_handle: TelegramRuntimeActorHandle,
    ) -> Result<(), TelegramError> {
        let mut actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        actors.insert(account_id, actor_handle);
        Ok(())
    }

    fn actor_command_tx(
        &self,
        account_id: &str,
    ) -> Result<Option<Sender<TelegramRuntimeCommand>>, TelegramError> {
        let actors = self.actors.lock().map_err(|_| {
            TelegramError::TdlibRuntime("Telegram runtime state lock poisoned".into())
        })?;
        Ok(actors
            .get(account_id)
            .and_then(|handle| handle.command_tx.clone()))
    }

    async fn ensure_tdlib_actor(
        &self,
        communication_store: &CommunicationIngestionStore,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        account: &ProviderAccount,
    ) -> Result<Sender<TelegramRuntimeCommand>, TelegramError> {
        if let Some(command_tx) = self.actor_command_tx(&account.account_id)? {
            return Ok(command_tx);
        }

        let session_encryption_key = optional_telegram_session_key(
            communication_store,
            secret_store,
            secret_resolver,
            &account.account_id,
        )
        .await?;
        let command_tx =
            spawn_tdlib_actor(config.clone(), account.clone(), session_encryption_key)?;
        self.set_actor_handle(
            account.account_id.clone(),
            TelegramRuntimeActorHandle {
                state: TelegramRuntimeActorState {
                    status: TelegramRuntimeState::Running,
                    last_error: None,
                    updated_at: Utc::now(),
                },
                command_tx: Some(command_tx.clone()),
            },
        )?;
        Ok(command_tx)
    }
}
