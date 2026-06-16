use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::integrations::telegram::client::{TelegramChatKind, TelegramDeliveryState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibTopicSnapshot {
    pub(crate) provider_topic_id: i64,
    pub(crate) title: String,
    pub(crate) icon_emoji: Option<String>,
    pub(crate) is_pinned: bool,
    pub(crate) is_closed: bool,
    pub(crate) unread_count: i64,
    pub(crate) last_message_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) chat_kind: TelegramChatKind,
    pub(crate) title: String,
    pub(crate) username: Option<String>,
    pub(crate) last_message_at: Option<DateTime<Utc>>,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibMessageSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) sender_id: String,
    pub(crate) sender_display_name: String,
    pub(crate) text: String,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) delivery_state: TelegramDeliveryState,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibFileSnapshot {
    pub(crate) file_id: i64,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) expected_size_bytes: Option<i64>,
    pub(crate) local_path: Option<String>,
    pub(crate) is_downloading_active: bool,
    pub(crate) is_downloading_completed: bool,
    pub(crate) downloaded_size_bytes: Option<i64>,
    pub(crate) remote_id: Option<String>,
    pub(crate) remote_unique_id: Option<String>,
    pub(crate) raw: Value,
}
