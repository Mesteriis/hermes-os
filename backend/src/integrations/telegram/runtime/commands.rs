use std::sync::mpsc::{self, Sender};

use tokio::task;

use crate::integrations::telegram::client::{TelegramError, TelegramManualSendRequest};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot,
    TelegramTdlibMessageSnapshot, TelegramTdlibTopicSnapshot,
};

use super::TDJSON_COMMAND_TIMEOUT;
use super::models::{TelegramHistorySyncMode, TelegramMediaSendRequest};
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

pub(super) async fn request_actor_chat_folders(
    command_tx: Sender<TelegramRuntimeCommand>,
    folder_ids: Vec<i64>,
) -> Result<Vec<TelegramTdlibChatFolderSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetChatFolders {
                folder_ids,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting folder sync commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib folder sync timed out".to_owned())
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

pub(super) async fn request_actor_send_media(
    command_tx: Sender<TelegramRuntimeCommand>,
    request: TelegramMediaSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SendMedia { request, reply_tx })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting media send commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib media send timed out".to_owned())
        })?
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

pub(super) async fn request_actor_forward(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    from_provider_chat_id: String,
    from_provider_message_id: String,
    command_id: String,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::ForwardMessage {
                provider_chat_id,
                from_provider_chat_id,
                from_provider_message_id,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting forward commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib forward timed out".to_owned())
        })?
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

pub(super) async fn request_actor_toggle_chat_unread(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    is_marked_as_unread: bool,
    read_through_provider_message_id: Option<String>,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::ToggleChatUnread {
                provider_chat_id,
                is_marked_as_unread,
                read_through_provider_message_id,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat unread commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat unread command timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_toggle_chat_archive(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    archived: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::ToggleChatArchive {
                provider_chat_id,
                archived,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat archive commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat archive command timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_toggle_chat_mute(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    muted: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::ToggleChatMute {
                provider_chat_id,
                muted,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat mute commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat mute command timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_add_chat_to_folder(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_folder_id: i64,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::AddChatToFolder {
                provider_chat_id,
                provider_folder_id,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat folder commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat folder command timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_remove_chat_from_folder(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_folder_id: i64,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::RemoveChatFromFolder {
                provider_chat_id,
                provider_folder_id,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat folder commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat folder command timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_join_chat(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::JoinChat {
                provider_chat_id,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat join commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat join command timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_leave_chat(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::LeaveChat {
                provider_chat_id,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat leave commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat leave command timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_search_messages(
    command_tx: Sender<TelegramRuntimeCommand>,
    query: String,
    limit: i32,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SearchMessages {
                query,
                limit,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting search commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib search timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_search_chat_messages(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    query: String,
    limit: i32,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::SearchChatMessages {
                provider_chat_id,
                query,
                limit,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting chat search commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib chat search timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_get_forum_topics(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    limit: i32,
) -> Result<Vec<TelegramTdlibTopicSnapshot>, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::GetForumTopics {
                provider_chat_id,
                limit,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting forum topic requests".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib forum topics timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_create_forum_topic(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    title: String,
    command_id: String,
) -> Result<TelegramTdlibTopicSnapshot, TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::CreateForumTopic {
                provider_chat_id,
                title,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting forum topic create commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime("Telegram TDLib forum topic create timed out".to_owned())
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}

pub(super) async fn request_actor_toggle_forum_topic_closed(
    command_tx: Sender<TelegramRuntimeCommand>,
    provider_chat_id: String,
    provider_topic_id: i64,
    is_closed: bool,
    command_id: String,
) -> Result<(), TelegramError> {
    task::spawn_blocking(move || {
        let (reply_tx, reply_rx) = mpsc::channel();
        command_tx
            .send(TelegramRuntimeCommand::ToggleForumTopicClosed {
                provider_chat_id,
                provider_topic_id,
                is_closed,
                command_id,
                reply_tx,
            })
            .map_err(|_| {
                TelegramError::TdlibRuntime(
                    "Telegram TDLib actor is not accepting forum topic close commands".to_owned(),
                )
            })?;
        reply_rx.recv_timeout(TDJSON_COMMAND_TIMEOUT).map_err(|_| {
            TelegramError::TdlibRuntime(
                "Telegram TDLib forum topic close command timed out".to_owned(),
            )
        })?
    })
    .await
    .map_err(|error| TelegramError::TdlibRuntime(format!("Telegram actor task failed: {error}")))?
}
