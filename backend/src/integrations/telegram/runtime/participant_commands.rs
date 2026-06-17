use std::sync::mpsc::{self, Sender};

use tokio::task;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::TelegramTdlibChatMemberSnapshot;

use super::TDJSON_COMMAND_TIMEOUT;
use super::state::TelegramRuntimeCommand;

pub(super) async fn request_actor_get_supergroup_members(
    command_tx: Sender<TelegramRuntimeCommand>,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetSupergroupMembers {
                supergroup_id,
                limit,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting member roster requests".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib member roster timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_get_supergroup_administrators(
    command_tx: Sender<TelegramRuntimeCommand>,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetSupergroupAdministrators {
                supergroup_id,
                limit,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting supergroup administrator sync commands"
                        .to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime(
                "Telegram TDLib supergroup administrator sync timed out".to_owned(),
            )
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_get_basic_group_members(
    command_tx: Sender<TelegramRuntimeCommand>,
    basic_group_id: i64,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetBasicGroupMembers {
                basic_group_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting basic-group roster requests".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib basic-group roster timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}
