use std::sync::mpsc::{self, Sender};

use tokio::task;

use crate::integrations::telegram::client::{TelegramError, TelegramManualSendRequest};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot, TelegramTdlibMessageSnapshot,
};

use super::TDJSON_COMMAND_TIMEOUT;
use super::models::TelegramHistorySyncMode;
use super::state::TelegramRuntimeCommand;

pub(super) async fn request_actor_chats(
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

pub(super) async fn request_actor_history(
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

pub(super) async fn request_actor_send(
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

pub(super) async fn request_actor_download_file(
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
