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

pub(super) async fn request_actor_edit_message(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_message_id: String,
    new_text: String,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::EditMessage {
                provider_chat_id,
                provider_message_id,
                new_text,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting edit commands".to_owned(),
                )
            })?;
        reply_rx
            .recv_timeout(TDJSON_COMMAND_TIMEOUT)
            .map_err(|_| TelegramError::TdlibRuntime("Telegram TDLib edit timed out".to_owned()))?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_delete_message(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_message_id: String,
    revoke: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::DeleteMessage {
                provider_chat_id,
                provider_message_id,
                revoke,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting delete commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib delete timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_set_reaction(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_message_id: String,
    reaction_emoji: String,
    is_active: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SetReaction {
                provider_chat_id,
                provider_message_id,
                reaction_emoji,
                is_active,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting reaction commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib reaction timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_reply(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    reply_to_provider_message_id: String,
    text: String,
    command_id: String,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::ReplyMessage {
                provider_chat_id,
                reply_to_provider_message_id,
                text,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting reply commands".to_owned(),
                )
            })?;
        reply_rx
            .recv_timeout(TDJSON_COMMAND_TIMEOUT)
            .map_err(|_| TelegramError::TdlibRuntime("Telegram TDLib reply timed out".to_owned()))?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_pin_message(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_message_id: String,
    pin: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::PinMessage {
                provider_chat_id,
                provider_message_id,
                pin,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting pin commands".to_owned(),
                )
            })?;
        reply_rx
            .recv_timeout(TDJSON_COMMAND_TIMEOUT)
            .map_err(|_| TelegramError::TdlibRuntime("Telegram TDLib pin timed out".to_owned()))?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}
