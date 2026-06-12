use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::task;

use crate::domains::mail::background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use crate::domains::mail::core::{
    CommunicationIngestionStore, CommunicationProviderKind, ProviderAccount,
    ProviderAccountSecretPurpose, ProviderCredentialError, ProviderCredentialReader,
};
use crate::domains::mail::storage::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanner, LocalMailBlobStore,
    MailAttachmentDisposition, MailStorageStore, NewMailAttachment, NewMailBlob,
    NoopAttachmentSafetyScanner,
};
use crate::integrations::telegram::client::{
    TelegramChat, TelegramError, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramMessage, TelegramQrLoginStartRequest, TelegramStore, telegram_text_preview_hash,
};
use crate::integrations::telegram::tdjson::{
    self, TdJsonClient, TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot,
    TelegramTdlibMessageSnapshot,
};
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

const TDJSON_BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(30);
const TDJSON_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const TDJSON_RECEIVE_POLL_SECONDS: f64 = 1.0;

#[derive(Clone, Default)]
pub struct TelegramRuntimeManager {
    actors: Arc<Mutex<HashMap<String, TelegramRuntimeActorHandle>>>,
}

pub(crate) struct TelegramMediaDownloadContext<'a, S: SecretResolver + ?Sized> {
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
        secret_resolver: &(impl SecretResolver + ?Sized),
        config: &AppConfig,
        request: &TelegramRuntimeStartRequest,
    ) -> Result<TelegramRuntimeStatus, TelegramError> {
        request.validate()?;
        let account = load_telegram_account(communication_store, &request.account_id).await?;
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
        secret_resolver: &(impl SecretResolver + ?Sized),
        config: &AppConfig,
        request: &TelegramChatSyncRequest,
    ) -> Result<TelegramChatSyncResponse, TelegramError> {
        request.validate()?;
        let account = load_telegram_account(communication_store, &request.account_id).await?;
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
        secret_resolver: &(impl SecretResolver + ?Sized),
        config: &AppConfig,
        request: &TelegramHistorySyncRequest,
    ) -> Result<TelegramHistorySyncResponse, TelegramError> {
        request.validate()?;
        let account = load_telegram_account(communication_store, &request.account_id).await?;
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
        secret_resolver: &(impl SecretResolver + ?Sized),
        config: &AppConfig,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError> {
        request.validate()?;
        let account = load_telegram_account(communication_store, &request.account_id).await?;
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

    pub(crate) async fn download_media<S: SecretResolver + ?Sized>(
        &self,
        context: TelegramMediaDownloadContext<'_, S>,
        request: &TelegramMediaDownloadRequest,
    ) -> Result<TelegramMediaDownloadResponse, TelegramError> {
        request.validate()?;
        let account =
            load_telegram_account(context.communication_store, &request.account_id).await?;
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
        secret_resolver: &(impl SecretResolver + ?Sized),
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramRuntimeStartRequest {
    pub account_id: String,
}

impl TelegramRuntimeStartRequest {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramChatSyncRequest {
    pub account_id: String,
    pub limit: Option<i64>,
}

impl TelegramChatSyncRequest {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        if let Some(limit) = self.limit {
            validate_limit(limit)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatSyncResponse {
    pub account_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub synced_count: usize,
    pub items: Vec<TelegramChat>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramHistorySyncRequest {
    pub account_id: String,
    pub provider_chat_id: String,
    pub from_message_id: Option<i64>,
    pub mode: Option<TelegramHistorySyncMode>,
    pub limit: Option<i64>,
}

impl TelegramHistorySyncRequest {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        if let Some(from_message_id) = self.from_message_id {
            if from_message_id <= 0 {
                return Err(TelegramError::InvalidRequest(
                    "from_message_id must be a positive TDLib message id".to_owned(),
                ));
            }
        }
        if self.mode() == TelegramHistorySyncMode::Older && self.from_message_id.is_none() {
            return Err(TelegramError::InvalidRequest(
                "from_message_id is required when mode=older".to_owned(),
            ));
        }
        if let Some(limit) = self.limit {
            validate_limit(limit)?;
        }
        Ok(())
    }

    fn mode(&self) -> TelegramHistorySyncMode {
        self.mode.unwrap_or(TelegramHistorySyncMode::Latest)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramHistorySyncMode {
    Latest,
    Older,
    Full,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramHistorySyncResponse {
    pub account_id: String,
    pub provider_chat_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub synced_count: usize,
    pub has_more: bool,
    pub next_from_message_id: Option<i64>,
    pub items: Vec<TelegramMessage>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramMediaDownloadRequest {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub tdlib_file_id: i64,
    pub provider_attachment_id: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub priority: Option<i32>,
}

impl TelegramMediaDownloadRequest {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        if self.tdlib_file_id <= 0 {
            return Err(TelegramError::InvalidRequest(
                "tdlib_file_id must be a positive TDLib file id".to_owned(),
            ));
        }
        if let Some(priority) = self.priority
            && !(1..=32).contains(&priority)
        {
            return Err(TelegramError::InvalidRequest(
                "priority must be between 1 and 32".to_owned(),
            ));
        }
        if let Some(value) = &self.provider_attachment_id {
            validate_non_empty("provider_attachment_id", value)?;
        }
        if let Some(value) = &self.filename {
            validate_non_empty("filename", value)?;
        }
        if let Some(value) = &self.content_type {
            validate_non_empty("content_type", value)?;
        }
        Ok(())
    }

    fn provider_attachment_id(&self) -> String {
        self.provider_attachment_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("tdlib-file:{}", self.tdlib_file_id))
    }

    fn content_type(&self) -> String {
        self.content_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "application/octet-stream".to_owned())
    }

    fn filename(&self) -> Option<String> {
        self.filename
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMediaDownloadResponse {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub tdlib_file_id: i64,
    pub local_path: Option<String>,
    pub size_bytes: Option<i64>,
    pub expected_size_bytes: Option<i64>,
    pub downloaded_size_bytes: Option<i64>,
    pub is_downloading_active: bool,
    pub is_downloading_completed: bool,
    pub attachment_id: Option<String>,
    pub blob_id: Option<String>,
    pub scan_status: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub status: String,
    pub fixture_runtime: bool,
    pub tdjson_runtime_available: bool,
    pub telegram_app_credentials_configured: bool,
    pub live_send_available: bool,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TelegramRuntimeState {
    Stopped,
    Running,
    Blocked,
    Degraded,
    Error,
}

impl TelegramRuntimeState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Running => "running",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TelegramRuntimeActorState {
    status: TelegramRuntimeState,
    last_error: Option<String>,
    updated_at: DateTime<Utc>,
}

impl TelegramRuntimeActorState {
    fn with_command(
        self,
        command_tx: Sender<TelegramRuntimeCommand>,
    ) -> (
        TelegramRuntimeActorState,
        Option<Sender<TelegramRuntimeCommand>>,
    ) {
        (self, Some(command_tx))
    }

    fn without_command(
        self,
    ) -> (
        TelegramRuntimeActorState,
        Option<Sender<TelegramRuntimeCommand>>,
    ) {
        (self, None)
    }
}

#[derive(Clone)]
struct TelegramRuntimeActorHandle {
    state: TelegramRuntimeActorState,
    command_tx: Option<Sender<TelegramRuntimeCommand>>,
}

enum TelegramRuntimeCommand {
    LoadChats {
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatSnapshot>, TelegramError>>,
    },
    SyncHistory {
        provider_chat_id: String,
        from_message_id: Option<i64>,
        limit: i32,
        mode: TelegramHistorySyncMode,
        reply_tx: Sender<Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError>>,
    },
    SendText {
        request: TelegramManualSendRequest,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
    DownloadFile {
        file_id: i64,
        priority: i32,
        reply_tx: Sender<Result<TelegramTdlibFileSnapshot, TelegramError>>,
    },
}

async fn request_actor_chats(
    command_tx: Sender<TelegramRuntimeCommand>,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::LoadChats { limit, reply_tx })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat sync commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat sync timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

async fn request_actor_history(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    from_message_id: Option<i64>,
    limit: i32,
    mode: TelegramHistorySyncMode,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SyncHistory {
                provider_chat_id,
                from_message_id,
                limit,
                mode,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting history sync commands".to_owned(),
                )
            })?;
        let timeout = if mode == TelegramHistorySyncMode::Full {
            TDJSON_COMMAND_TIMEOUT * 10
        } else {
            TDJSON_COMMAND_TIMEOUT
        };
        reply_rx.recv_timeout(timeout).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib history sync timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

async fn request_actor_send(
    command_tx: Sender<TelegramRuntimeCommand>,
    request: TelegramManualSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SendText { request, reply_tx })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting send commands".to_owned(),
                )
            })?;
        reply_rx
            .recv_timeout(TDJSON_COMMAND_TIMEOUT)
            .map_err(|_| TelegramError::TdlibRuntime("Telegram TDLib send timed out".to_owned()))?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

async fn request_actor_download_file(
    command_tx: Sender<TelegramRuntimeCommand>,
    file_id: i64,
    priority: i32,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::DownloadFile {
                file_id,
                priority,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting media download commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib media download timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

async fn persist_downloaded_media(
    telegram_store: &TelegramStore,
    mail_store: &MailStorageStore,
    request: &TelegramMediaDownloadRequest,
    file: &TelegramTdlibFileSnapshot,
    blob_root: &Path,
) -> Result<TelegramMediaDownloadResponse, TelegramError> {
    let mut response = TelegramMediaDownloadResponse {
        account_id: request.account_id.trim().to_owned(),
        provider_chat_id: request.provider_chat_id.trim().to_owned(),
        provider_message_id: request.provider_message_id.trim().to_owned(),
        runtime_kind: "tdlib_qr_authorized".to_owned(),
        status: if file.is_downloading_completed {
            "downloaded".to_owned()
        } else if file.is_downloading_active {
            "downloading".to_owned()
        } else {
            "remote".to_owned()
        },
        tdlib_file_id: file.file_id,
        local_path: file.local_path.clone(),
        size_bytes: file.size_bytes,
        expected_size_bytes: file.expected_size_bytes,
        downloaded_size_bytes: file.downloaded_size_bytes,
        is_downloading_active: file.is_downloading_active,
        is_downloading_completed: file.is_downloading_completed,
        attachment_id: None,
        blob_id: None,
        scan_status: None,
    };

    if !file.is_downloading_completed {
        return Ok(response);
    }

    let local_path = file.local_path.as_deref().ok_or_else(|| {
        TelegramError::TdlibRuntime(
            "TDLib reported a completed download without a local file path".to_owned(),
        )
    })?;
    let bytes = tokio::fs::read(local_path).await.map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to read downloaded Telegram file `{local_path}`: {error}"
        ))
    })?;
    let blob_store = LocalMailBlobStore::new(blob_root);
    let local_blob = blob_store.put_blob(&bytes).await.map_err(|error| {
        TelegramError::TdlibRuntime(format!("failed to store Telegram media blob: {error}"))
    })?;
    let content_type = request.content_type();
    let stored_blob = mail_store
        .upsert_blob(&NewMailBlob::from_local_blob(&local_blob).content_type(content_type.clone()))
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("failed to record Telegram media blob: {error}"))
        })?;
    let scanner = NoopAttachmentSafetyScanner;
    let provider_attachment_id = request.provider_attachment_id();
    let filename = request.filename();
    let scan_report = scanner
        .scan(&AttachmentSafetyScanRequest {
            provider_attachment_id: &provider_attachment_id,
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &local_blob.sha256,
            storage_kind: &local_blob.storage_kind,
            storage_path: &local_blob.storage_path,
            bytes: &bytes,
        })
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("Telegram media scan failed: {error}"))
        })?;
    let anchor = telegram_store
        .attachment_anchor_for_message(
            &request.account_id,
            &request.provider_chat_id,
            &request.provider_message_id,
        )
        .await?;
    let mut attachment = NewMailAttachment::new(
        anchor.message_id,
        anchor.raw_record_id,
        stored_blob.blob_id.clone(),
        provider_attachment_id,
        content_type,
        local_blob.size_bytes,
        local_blob.sha256.clone(),
    )
    .disposition(MailAttachmentDisposition::Attachment)
    .scan_report(scan_report);
    if let Some(filename) = filename {
        attachment = attachment.filename(filename);
    }
    let stored_attachment = mail_store
        .upsert_attachment(&attachment)
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!(
                "failed to record Telegram media attachment: {error}"
            ))
        })?;

    response.attachment_id = Some(stored_attachment.attachment_id);
    response.blob_id = Some(stored_blob.blob_id);
    response.scan_status = Some(stored_attachment.scan_status.as_str().to_owned());
    Ok(response)
}

fn spawn_tdlib_actor(
    config: AppConfig,
    account: ProviderAccount,
    session_encryption_key: Option<String>,
) -> Result<Sender<TelegramRuntimeCommand>, TelegramError> {
    if !tdjson::runtime_available(config.tdjson_path()) {
        return Err(TelegramError::TdlibRuntimeUnavailable(
            "libtdjson is not available for Telegram live runtime".to_owned(),
        ));
    }
    let start_request =
        tdlib_start_request_from_account(&config, &account, session_encryption_key)?;
    let (command_tx, command_rx) = mpsc::channel();
    let thread_name = format!(
        "telegram-tdlib-{}",
        short_thread_suffix(&account.account_id)
    );
    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            if let Err(error) = drive_tdlib_actor(config, start_request, command_rx) {
                tracing::warn!(error = %error, "Telegram TDLib actor stopped");
            }
        })
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!("failed to spawn Telegram TDLib actor: {error}"))
        })?;

    Ok(command_tx)
}

fn tdlib_start_request_from_account(
    config: &AppConfig,
    account: &ProviderAccount,
    session_encryption_key: Option<String>,
) -> Result<TelegramQrLoginStartRequest, TelegramError> {
    let api_id = config.telegram_api_id().ok_or_else(|| {
        TelegramError::InvalidRequest(
            "HERMES_TELEGRAM_API_ID is required for Telegram TDLib runtime".to_owned(),
        )
    })?;
    let api_hash = config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
        .ok_or_else(|| {
            TelegramError::InvalidRequest(
                "HERMES_TELEGRAM_API_HASH is required for Telegram TDLib runtime".to_owned(),
            )
        })?;
    let tdlib_data_path = account
        .config
        .get("tdlib_data_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(
                "tdlib_data_path is required for Telegram TDLib runtime".to_owned(),
            )
        })?;

    Ok(TelegramQrLoginStartRequest {
        account_id: account.account_id.clone(),
        display_name: account.display_name.clone(),
        external_account_id: account.external_account_id.clone(),
        api_id: Some(api_id),
        api_hash: Some(api_hash),
        session_encryption_key,
        tdlib_data_path: Some(tdlib_data_path),
        transcription_enabled: false,
    })
}

async fn optional_telegram_session_key(
    communication_store: &CommunicationIngestionStore,
    secret_store: &SecretReferenceStore,
    secret_resolver: &(impl SecretResolver + ?Sized),
    account_id: &str,
) -> Result<Option<String>, TelegramError> {
    let credential_reader = ProviderCredentialReader::new(
        communication_store.clone(),
        secret_store.clone(),
        secret_resolver,
    );
    match credential_reader
        .read(account_id, ProviderAccountSecretPurpose::TelegramSessionKey)
        .await
    {
        Ok(credential) => Ok(Some(credential.secret.expose_for_runtime().to_owned())),
        Err(ProviderCredentialError::MissingBinding { .. }) => Ok(None),
        Err(error) => Err(TelegramError::TdlibRuntime(format!(
            "failed to resolve Telegram session encryption key: {error}"
        ))),
    }
}

fn drive_tdlib_actor(
    config: AppConfig,
    start_request: TelegramQrLoginStartRequest,
    command_rx: mpsc::Receiver<TelegramRuntimeCommand>,
) -> Result<(), TelegramError> {
    let library = tdjson::TdJsonLibrary::load(config.tdjson_path())?;
    let client = library.create_client()?;
    prepare_tdlib_client(&client, &start_request)?;
    wait_for_tdlib_ready(&client, &start_request)?;

    while let Ok(command) = command_rx.recv() {
        match command {
            TelegramRuntimeCommand::LoadChats { limit, reply_tx } => {
                let _ = reply_tx.send(actor_load_chats(&client, limit));
            }
            TelegramRuntimeCommand::SyncHistory {
                provider_chat_id,
                from_message_id,
                limit,
                mode,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_sync_history(
                    &client,
                    &provider_chat_id,
                    from_message_id,
                    limit,
                    mode,
                ));
            }
            TelegramRuntimeCommand::SendText { request, reply_tx } => {
                let _ = reply_tx.send(actor_send_text(&client, &request));
            }
            TelegramRuntimeCommand::DownloadFile {
                file_id,
                priority,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_download_file(&client, file_id, priority));
            }
        }
    }

    let _ = client.send_json(&json!({ "@type": "close" }));
    Ok(())
}

fn prepare_tdlib_client(
    client: &TdJsonClient,
    start_request: &TelegramQrLoginStartRequest,
) -> Result<(), TelegramError> {
    let database_directory = tdjson::tdlib_database_directory(start_request);
    let files_directory = database_directory.join("files");
    std::fs::create_dir_all(&files_directory).map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to create TDLib data directory `{}`: {error}",
            files_directory.display()
        ))
    })?;
    let _ = client.execute_json(&json!({
        "@type": "setLogVerbosityLevel",
        "new_verbosity_level": 1
    }));
    client.send_json(&json!({
        "@type": "getAuthorizationState",
        "@extra": "hermes-runtime-initial-authorization-state"
    }))?;
    Ok(())
}

fn wait_for_tdlib_ready(
    client: &TdJsonClient,
    start_request: &TelegramQrLoginStartRequest,
) -> Result<(), TelegramError> {
    let database_directory = tdjson::tdlib_database_directory(start_request);
    let started_at = Instant::now();
    let mut tdlib_parameters_sent = false;

    while started_at.elapsed() < TDJSON_BOOTSTRAP_TIMEOUT {
        let Some(event) = client.receive_json(TDJSON_RECEIVE_POLL_SECONDS)? else {
            continue;
        };

        if tdjson::is_tdlib_parameters_not_specified_error(&event) && !tdlib_parameters_sent {
            client.send_json(&tdjson::set_tdlib_parameters_request(
                start_request,
                &database_directory,
            )?)?;
            tdlib_parameters_sent = true;
            continue;
        }
        if tdjson::is_tdlib_database_encryption_key_needed_error(&event) {
            client.send_json(&tdjson::check_database_encryption_key_request(
                start_request,
            ))?;
            continue;
        }
        if let Some(message) = tdjson::tdlib_error_message(&event) {
            return Err(TelegramError::TdlibRuntime(message));
        }

        let Some(authorization_state) = tdjson::authorization_state(&event) else {
            continue;
        };
        match authorization_state.get("@type").and_then(Value::as_str) {
            Some("authorizationStateWaitTdlibParameters") => {
                client.send_json(&tdjson::set_tdlib_parameters_request(
                    start_request,
                    &database_directory,
                )?)?;
                tdlib_parameters_sent = true;
            }
            Some("authorizationStateWaitEncryptionKey") => {
                client.send_json(&tdjson::check_database_encryption_key_request(
                    start_request,
                ))?;
            }
            Some("authorizationStateReady") => return Ok(()),
            Some("authorizationStateClosed")
            | Some("authorizationStateClosing")
            | Some("authorizationStateLoggingOut") => {
                return Err(TelegramError::TdlibRuntime(
                    "Telegram TDLib authorization session is closed".to_owned(),
                ));
            }
            Some(wait_state) if wait_state.starts_with("authorizationStateWait") => {
                return Err(TelegramError::TdlibRuntime(format!(
                    "Telegram TDLib account is not authorized; current state is `{wait_state}`"
                )));
            }
            _ => {}
        }
    }

    Err(TelegramError::TdlibRuntime(
        "Telegram TDLib authorization did not become ready in time".to_owned(),
    ))
}

fn actor_load_chats(
    client: &TdJsonClient,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatSnapshot>, TelegramError> {
    let load_extra = "hermes-runtime-load-chats";
    client.send_json(&tdjson::tdlib_load_chats_request(limit, load_extra))?;
    let load_response = receive_tdlib_extra(client, load_extra, TDJSON_COMMAND_TIMEOUT)?;
    if tdjson::tdlib_error_message(&load_response).is_some() && !is_tdlib_not_found(&load_response)
    {
        return Err(TelegramError::TdlibRuntime(
            tdjson::tdlib_error_message(&load_response)
                .unwrap_or_else(|| "TDLib loadChats failed".to_owned()),
        ));
    }

    let chats_extra = "hermes-runtime-get-chats";
    client.send_json(&tdjson::tdlib_get_chats_request(limit, chats_extra))?;
    let chats_response = receive_tdlib_extra(client, chats_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&chats_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    let chat_ids = tdjson::parse_tdlib_chat_ids(&chats_response)?;
    let mut snapshots = Vec::with_capacity(chat_ids.len());
    for chat_id in chat_ids {
        let extra = format!("hermes-runtime-get-chat-{chat_id}");
        client.send_json(&tdjson::tdlib_get_chat_request(chat_id, &extra))?;
        let chat_response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&chat_response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        snapshots.push(tdjson::parse_tdlib_chat_snapshot(&chat_response)?);
    }
    Ok(snapshots)
}

fn actor_sync_history(
    client: &TdJsonClient,
    provider_chat_id: &str,
    from_message_id: Option<i64>,
    limit: i32,
    mode: TelegramHistorySyncMode,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let page_limit = limit.clamp(1, 100);
    let mut cursor = from_message_id;
    let mut snapshots = Vec::new();
    let mut page_index = 0;

    loop {
        let extra = format!(
            "hermes-runtime-history-{chat_id}-{}-{page_index}",
            cursor.unwrap_or(0)
        );
        client.send_json(&tdjson::tdlib_get_chat_history_request(
            chat_id, cursor, page_limit, false, &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        let page = tdjson::parse_tdlib_message_list(&response)?;
        if page.is_empty() {
            break;
        }

        let page_len = page.len();
        let next_cursor = oldest_tdlib_message_id(&page);
        snapshots.extend(page);
        if mode != TelegramHistorySyncMode::Full || page_len < page_limit as usize {
            break;
        }
        if next_cursor.is_none() || next_cursor == cursor {
            break;
        }
        cursor = next_cursor;
        page_index += 1;
    }

    Ok(snapshots)
}

fn actor_send_text(
    client: &TdJsonClient,
    request: &TelegramManualSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(&request.provider_chat_id)?;
    let extra = format!("hermes-runtime-send-{}", request.command_id.trim());
    client.send_json(&tdjson::tdlib_send_text_message_request(
        chat_id,
        &request.text,
        &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_snapshot(&response)
}

fn actor_download_file(
    client: &TdJsonClient,
    file_id: i64,
    priority: i32,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    let extra = format!("hermes-runtime-download-file-{file_id}");
    client.send_json(&tdjson::tdlib_download_file_request(
        file_id, priority, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_file_snapshot(&response)
}

fn receive_tdlib_extra(
    client: &TdJsonClient,
    expected_extra: &str,
    timeout: Duration,
) -> Result<Value, TelegramError> {
    let started_at = Instant::now();
    while started_at.elapsed() < timeout {
        let Some(event) = client.receive_json(TDJSON_RECEIVE_POLL_SECONDS)? else {
            continue;
        };
        if event.get("@extra").and_then(Value::as_str) == Some(expected_extra) {
            return Ok(event);
        }
        if let Some(message) = tdjson::tdlib_error_message(&event) {
            tracing::debug!(error = %message, "ignored unrelated TDLib error while waiting for correlated response");
        }
    }
    Err(TelegramError::TdlibRuntime(format!(
        "TDLib request `{expected_extra}` timed out"
    )))
}

fn oldest_tdlib_message_id(snapshots: &[TelegramTdlibMessageSnapshot]) -> Option<i64> {
    snapshots
        .iter()
        .filter_map(|snapshot| snapshot.provider_message_id.trim().parse::<i64>().ok())
        .min()
}

fn tdlib_provider_chat_id(provider_chat_id: &str) -> Result<i64, TelegramError> {
    provider_chat_id.trim().parse::<i64>().map_err(|_| {
        TelegramError::InvalidRequest(format!(
            "TDLib provider_chat_id `{}` must be a Telegram numeric chat id",
            provider_chat_id.trim()
        ))
    })
}

fn is_tdlib_not_found(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(404)
}

fn short_thread_suffix(account_id: &str) -> String {
    let sanitized = account_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();
    if sanitized.is_empty() {
        "account".to_owned()
    } else {
        sanitized.chars().take(32).collect()
    }
}

async fn load_telegram_account(
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

fn status_from_account(
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

fn account_runtime_kind(account: &ProviderAccount) -> String {
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

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, TelegramError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=100).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn history_sync_request_accepts_older_cursor() {
        let request: TelegramHistorySyncRequest = serde_json::from_value(json!({
            "account_id": "telegram-primary",
            "provider_chat_id": "-100123456789",
            "from_message_id": 987654321,
            "mode": "older",
            "limit": 100
        }))
        .expect("history request");

        request.validate().expect("valid history request");
        assert_eq!(request.mode(), TelegramHistorySyncMode::Older);
        assert_eq!(request.from_message_id, Some(987654321));
    }

    #[test]
    fn history_sync_response_exposes_next_cursor() {
        let response = TelegramHistorySyncResponse {
            account_id: "telegram-primary".to_owned(),
            provider_chat_id: "-100123456789".to_owned(),
            runtime_kind: "tdlib_qr_authorized".to_owned(),
            status: "synced".to_owned(),
            synced_count: 100,
            has_more: true,
            next_from_message_id: Some(12345),
            items: Vec::new(),
        };

        let value = serde_json::to_value(response).expect("serialized response");
        assert_eq!(value["has_more"], json!(true));
        assert_eq!(value["next_from_message_id"], json!(12345));
    }
}
