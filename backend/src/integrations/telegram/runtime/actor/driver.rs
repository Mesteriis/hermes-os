use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};
use crate::integrations::telegram::tdjson::{self, TdJsonClient};
use crate::platform::config::AppConfig;

use super::super::state::{TelegramRuntimeCommand, TelegramRuntimeEvent};
use super::authorization::{prepare_tdlib_client, wait_for_tdlib_ready};
use super::chats::{actor_get_chat_folders, actor_load_chats};
use super::download::actor_download_file;
use super::edit::{
    actor_add_chat_to_folder, actor_delete_message, actor_edit_message, actor_join_chat,
    actor_leave_chat, actor_pin_message, actor_remove_chat_from_folder, actor_set_reaction,
    actor_toggle_chat_archive, actor_toggle_chat_mute, actor_toggle_chat_unread,
};
use super::history::actor_sync_history;
use super::participants::{
    actor_get_basic_group_members, actor_get_supergroup_administrators,
    actor_get_supergroup_members,
};
use super::search::{actor_search_chat_messages, actor_search_messages};
use super::send::{actor_send_forward, actor_send_media, actor_send_reply, actor_send_text};
use super::topics::{
    actor_create_forum_topic, actor_get_forum_topics, actor_toggle_forum_topic_closed,
};

pub(super) fn drive_tdlib_actor(
    config: AppConfig,
    start_request: TelegramQrLoginStartRequest,
    command_rx: mpsc::Receiver<TelegramRuntimeCommand>,
    runtime_event_tx: Option<UnboundedSender<TelegramRuntimeEvent>>,
) -> Result<(), TelegramError> {
    let library = tdjson::TdJsonLibrary::load(config.tdjson_path())?;
    let client = library.create_client()?;
    prepare_tdlib_client(&client, &start_request)?;
    wait_for_tdlib_ready(&client, &start_request)?;

    loop {
        let command = match command_rx.recv_timeout(Duration::from_millis(250)) {
            Ok(command) => command,
            Err(RecvTimeoutError::Timeout) => {
                drain_unsolicited_tdlib_events(&client, runtime_event_tx.as_ref())?;
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };

        match command {
            TelegramRuntimeCommand::LoadChats { limit, reply_tx } => {
                let _ = reply_tx.send(actor_load_chats(&client, limit));
            }
            TelegramRuntimeCommand::GetChatFolders {
                folder_ids,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_chat_folders(&client, &folder_ids));
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
            TelegramRuntimeCommand::SendMedia { request, reply_tx } => {
                let _ = reply_tx.send(actor_send_media(&client, &request));
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
            TelegramRuntimeCommand::ToggleChatUnread {
                provider_chat_id,
                is_marked_as_unread,
                read_through_provider_message_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_toggle_chat_unread(
                    &client,
                    &provider_chat_id,
                    is_marked_as_unread,
                    read_through_provider_message_id.as_deref(),
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ToggleChatArchive {
                provider_chat_id,
                archived,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_toggle_chat_archive(
                    &client,
                    &provider_chat_id,
                    archived,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ToggleChatMute {
                provider_chat_id,
                muted,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_toggle_chat_mute(
                    &client,
                    &provider_chat_id,
                    muted,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::AddChatToFolder {
                provider_chat_id,
                provider_folder_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_add_chat_to_folder(
                    &client,
                    &provider_chat_id,
                    provider_folder_id,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::RemoveChatFromFolder {
                provider_chat_id,
                provider_folder_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_remove_chat_from_folder(
                    &client,
                    &provider_chat_id,
                    provider_folder_id,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::JoinChat {
                provider_chat_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_join_chat(&client, &provider_chat_id, &command_id));
            }
            TelegramRuntimeCommand::LeaveChat {
                provider_chat_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_leave_chat(&client, &provider_chat_id, &command_id));
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
            TelegramRuntimeCommand::ForwardMessage {
                provider_chat_id,
                from_provider_chat_id,
                from_provider_message_id,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_send_forward(
                    &client,
                    &provider_chat_id,
                    &from_provider_chat_id,
                    &from_provider_message_id,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::GetForumTopics {
                provider_chat_id,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_forum_topics(&client, &provider_chat_id, limit));
            }
            TelegramRuntimeCommand::CreateForumTopic {
                provider_chat_id,
                title,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_create_forum_topic(
                    &client,
                    &provider_chat_id,
                    &title,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::ToggleForumTopicClosed {
                provider_chat_id,
                provider_topic_id,
                is_closed,
                command_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_toggle_forum_topic_closed(
                    &client,
                    &provider_chat_id,
                    provider_topic_id,
                    is_closed,
                    &command_id,
                ));
            }
            TelegramRuntimeCommand::GetSupergroupMembers {
                supergroup_id,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_supergroup_members(&client, supergroup_id, limit));
            }
            TelegramRuntimeCommand::GetSupergroupAdministrators {
                supergroup_id,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_supergroup_administrators(
                    &client,
                    supergroup_id,
                    limit,
                ));
            }
            TelegramRuntimeCommand::GetBasicGroupMembers {
                basic_group_id,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_get_basic_group_members(&client, basic_group_id));
            }
            TelegramRuntimeCommand::SearchMessages {
                query,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_search_messages(&client, &query, limit));
            }
            TelegramRuntimeCommand::SearchChatMessages {
                provider_chat_id,
                query,
                limit,
                reply_tx,
            } => {
                let _ = reply_tx.send(actor_search_chat_messages(
                    &client,
                    &provider_chat_id,
                    &query,
                    limit,
                ));
            }
        }
        drain_unsolicited_tdlib_events(&client, runtime_event_tx.as_ref())?;
    }

    let _ = client.send_json(&json!({ "@type": "close" }));
    Ok(())
}

fn drain_unsolicited_tdlib_events(
    client: &TdJsonClient,
    runtime_event_tx: Option<&UnboundedSender<TelegramRuntimeEvent>>,
) -> Result<(), TelegramError> {
    let Some(runtime_event_tx) = runtime_event_tx else {
        return Ok(());
    };

    while let Some(event) = client.receive_json(0.0)? {
        if let Some(snapshot) = tdjson::parse_tdlib_new_message_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::MessageCreated(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_message_content_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::MessageContentUpdated(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_message_edited_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::MessageEdited(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_message_pinned_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::MessagePinnedUpdated(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_message_send_failed_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::MessageSendFailed(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_message_send_succeeded_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::MessageSendSucceeded(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_message_delete_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::MessageDeleted(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_message_interaction_info_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::MessageInteractionInfoUpdated(
                snapshot,
            ));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_typing_snapshot(&event) {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::TypingChanged(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_topic_update_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::TopicUpdated(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_chat_unread_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::ChatUnreadUpdated(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_chat_marked_as_unread_snapshot(&event)? {
            let _ =
                runtime_event_tx.send(TelegramRuntimeEvent::ChatMarkedAsUnreadUpdated(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_chat_notification_settings_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::ChatNotificationSettingsUpdated(
                snapshot,
            ));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_chat_position_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::ChatPositionUpdated(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_chat_removed_from_list_snapshot(&event)? {
            let _ = runtime_event_tx.send(TelegramRuntimeEvent::ChatRemovedFromList(snapshot));
        }
        if let Some(snapshot) = tdjson::parse_tdlib_chat_folders_update_snapshot(&event)? {
            let _ =
                runtime_event_tx.send(TelegramRuntimeEvent::ChatFoldersUpdated(snapshot.folders));
        }
    }

    Ok(())
}
