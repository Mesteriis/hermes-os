use std::sync::mpsc::Sender;

use chrono::{DateTime, Utc};

use crate::integrations::telegram::client::{TelegramError, TelegramManualSendRequest};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatMarkedAsUnreadSnapshot,
    TelegramTdlibChatMemberSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatRemovedFromListSnapshot,
    TelegramTdlibChatSnapshot, TelegramTdlibChatUnreadSnapshot, TelegramTdlibFileSnapshot,
    TelegramTdlibMessageContentSnapshot, TelegramTdlibMessageDeleteSnapshot,
    TelegramTdlibMessageEditedSnapshot, TelegramTdlibMessageInteractionInfoSnapshot,
    TelegramTdlibMessagePinnedSnapshot, TelegramTdlibMessageSendFailedSnapshot,
    TelegramTdlibMessageSendSucceededSnapshot, TelegramTdlibMessageSnapshot,
    TelegramTdlibTopicSnapshot, TelegramTdlibTopicUpdateSnapshot, TelegramTdlibTypingSnapshot,
};

use super::models::{TelegramHistorySyncMode, TelegramMediaSendRequest};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TelegramRuntimeState {
    Stopped,
    Running,
    Blocked,
    Degraded,
    Error,
}

impl TelegramRuntimeState {
    pub(super) fn as_str(self) -> &'static str {
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
pub(super) struct TelegramRuntimeActorState {
    pub(super) status: TelegramRuntimeState,
    pub(super) last_error: Option<String>,
    pub(super) updated_at: DateTime<Utc>,
}

impl TelegramRuntimeActorState {
    pub(super) fn with_command(
        self,
        command_tx: Sender<TelegramRuntimeCommand>,
    ) -> (
        TelegramRuntimeActorState,
        Option<Sender<TelegramRuntimeCommand>>,
    ) {
        (self, Some(command_tx))
    }

    pub(super) fn without_command(
        self,
    ) -> (
        TelegramRuntimeActorState,
        Option<Sender<TelegramRuntimeCommand>>,
    ) {
        (self, None)
    }
}

#[derive(Clone)]
pub(super) struct TelegramRuntimeActorHandle {
    pub(super) state: TelegramRuntimeActorState,
    pub(super) command_tx: Option<Sender<TelegramRuntimeCommand>>,
}

pub(super) enum TelegramRuntimeCommand {
    LoadChats {
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatSnapshot>, TelegramError>>,
    },
    GetChatFolders {
        folder_ids: Vec<i64>,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatFolderSnapshot>, TelegramError>>,
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
    SendMedia {
        request: TelegramMediaSendRequest,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
    DownloadFile {
        file_id: i64,
        priority: i32,
        reply_tx: Sender<Result<TelegramTdlibFileSnapshot, TelegramError>>,
    },
    EditMessage {
        provider_chat_id: String,
        provider_message_id: String,
        new_text: String,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    DeleteMessage {
        provider_chat_id: String,
        provider_message_id: String,
        revoke: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    SetReaction {
        provider_chat_id: String,
        provider_message_id: String,
        reaction_emoji: String,
        is_active: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    PinMessage {
        provider_chat_id: String,
        provider_message_id: String,
        pin: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    ToggleChatUnread {
        provider_chat_id: String,
        is_marked_as_unread: bool,
        read_through_provider_message_id: Option<String>,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    ToggleChatArchive {
        provider_chat_id: String,
        archived: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    ToggleChatMute {
        provider_chat_id: String,
        muted: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    AddChatToFolder {
        provider_chat_id: String,
        provider_folder_id: i64,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    RemoveChatFromFolder {
        provider_chat_id: String,
        provider_folder_id: i64,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    JoinChat {
        provider_chat_id: String,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    LeaveChat {
        provider_chat_id: String,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    ReplyMessage {
        provider_chat_id: String,
        reply_to_provider_message_id: String,
        text: String,
        command_id: String,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
    ForwardMessage {
        provider_chat_id: String,
        from_provider_chat_id: String,
        from_provider_message_id: String,
        command_id: String,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
    GetForumTopics {
        provider_chat_id: String,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibTopicSnapshot>, TelegramError>>,
    },
    CreateForumTopic {
        provider_chat_id: String,
        title: String,
        command_id: String,
        reply_tx: Sender<Result<TelegramTdlibTopicSnapshot, TelegramError>>,
    },
    ToggleForumTopicClosed {
        provider_chat_id: String,
        provider_topic_id: i64,
        is_closed: bool,
        command_id: String,
        reply_tx: Sender<Result<(), TelegramError>>,
    },
    GetSupergroupMembers {
        supergroup_id: i64,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError>>,
    },
    GetSupergroupAdministrators {
        supergroup_id: i64,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError>>,
    },
    GetBasicGroupMembers {
        basic_group_id: i64,
        reply_tx: Sender<Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError>>,
    },
    SearchMessages {
        query: String,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError>>,
    },
    SearchChatMessages {
        provider_chat_id: String,
        query: String,
        limit: i32,
        reply_tx: Sender<Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum TelegramRuntimeEvent {
    MessageCreated(TelegramTdlibMessageSnapshot),
    MessageContentUpdated(TelegramTdlibMessageContentSnapshot),
    MessageEdited(TelegramTdlibMessageEditedSnapshot),
    MessagePinnedUpdated(TelegramTdlibMessagePinnedSnapshot),
    MessageSendFailed(TelegramTdlibMessageSendFailedSnapshot),
    MessageSendSucceeded(TelegramTdlibMessageSendSucceededSnapshot),
    MessageDeleted(TelegramTdlibMessageDeleteSnapshot),
    MessageInteractionInfoUpdated(TelegramTdlibMessageInteractionInfoSnapshot),
    TypingChanged(TelegramTdlibTypingSnapshot),
    TopicUpdated(TelegramTdlibTopicUpdateSnapshot),
    ChatUnreadUpdated(TelegramTdlibChatUnreadSnapshot),
    ChatMarkedAsUnreadUpdated(TelegramTdlibChatMarkedAsUnreadSnapshot),
    ChatNotificationSettingsUpdated(TelegramTdlibChatNotificationSettingsSnapshot),
    ChatPositionUpdated(TelegramTdlibChatPositionSnapshot),
    ChatRemovedFromList(TelegramTdlibChatRemovedFromListSnapshot),
    ChatFoldersUpdated(Vec<TelegramTdlibChatFolderSnapshot>),
}
