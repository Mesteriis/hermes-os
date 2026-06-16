use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::errors::TelegramError;
use super::super::validation::{validate_non_empty, validate_object};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChat {
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: String,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: String,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatMember {
    pub sender_id: String,
    pub sender_display_name: Option<String>,
    pub message_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatGroupFilter {
    pub id: String,
    pub label: String,
    pub source: String,
    pub count: i64,
    pub icon: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatGroupFilterListResponse {
    pub items: Vec<TelegramChatGroupFilter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTelegramChat {
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: TelegramChatKind,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: TelegramSyncState,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewTelegramChat {
    pub(in crate::integrations::telegram::client) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("title", &self.title)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramChatKind {
    Private,
    Group,
    Channel,
    Bot,
}

impl TelegramChatKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Group => "group",
            Self::Channel => "channel",
            Self::Bot => "bot",
        }
    }
}

impl TryFrom<&str> for TelegramChatKind {
    type Error = TelegramError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "private" => Ok(Self::Private),
            "group" => Ok(Self::Group),
            "channel" => Ok(Self::Channel),
            "bot" => Ok(Self::Bot),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram chat_kind `{other}`"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramSyncState {
    Fixture,
    Syncing,
    Synced,
    Degraded,
    Error,
}

impl TelegramSyncState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Syncing => "syncing",
            Self::Synced => "synced",
            Self::Degraded => "degraded",
            Self::Error => "error",
        }
    }
}
