use std::sync::mpsc::Sender;

use chrono::{DateTime, Utc};

use crate::integrations::telegram::client::{TelegramError, TelegramManualSendRequest};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot, TelegramTdlibMessageSnapshot,
};

use super::models::TelegramHistorySyncMode;

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
    ReplyMessage {
        provider_chat_id: String,
        reply_to_provider_message_id: String,
        text: String,
        command_id: String,
        reply_tx: Sender<Result<TelegramTdlibMessageSnapshot, TelegramError>>,
    },
}
