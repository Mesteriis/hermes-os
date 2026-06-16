use std::sync::mpsc;

use serde_json::json;

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};
use crate::integrations::telegram::tdjson;
use crate::platform::config::AppConfig;

use super::super::state::TelegramRuntimeCommand;
use super::authorization::{prepare_tdlib_client, wait_for_tdlib_ready};
use super::chats::actor_load_chats;
use super::download::actor_download_file;
use super::edit::{
    actor_delete_message, actor_edit_message, actor_pin_message, actor_set_reaction,
};
use super::history::actor_sync_history;
use super::send::{actor_send_reply, actor_send_text};

pub(super) fn drive_tdlib_actor(
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
            TelegramRuntimeCommand::EditMessage {
                provider_chat_id,
                provider_message_id,
                new_text,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_edit_message(
                    &client,
                    &provider_chat_id,
                    &provider_message_id,
                    &new_text,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::DeleteMessage {
                provider_chat_id,
                provider_message_id,
                revoke,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_delete_message(
                    &client,
                    &provider_chat_id,
                    &provider_message_id,
                    revoke,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::SetReaction {
                provider_chat_id,
                provider_message_id,
                reaction_emoji,
                is_active,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_set_reaction(
                    &client,
                    &provider_chat_id,
                    &provider_message_id,
                    &reaction_emoji,
                    is_active,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::PinMessage {
                provider_chat_id,
                provider_message_id,
                pin,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_pin_message(
                    &client,
                    &provider_chat_id,
                    &provider_message_id,
                    pin,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ReplyMessage {
                provider_chat_id,
                reply_to_provider_message_id,
                text,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_send_reply(
                    &client,
                    &provider_chat_id,
                    &reply_to_provider_message_id,
                    &text,
                    &command_id,
                ));
            }
        }
    }

    let _ = client.send_json(&json!({ "@type": "close" }));
    Ok(())
}
