use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::errors::TelegramError;
use super::super::validation::validate_non_empty;

#[derive(Clone, Debug, Serialize)]
pub struct TelegramTopic {
    pub topic_id: String,
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_topic_id: i64,
    pub provider_chat_id: String,
    pub title: String,
    pub icon_emoji: Option<String>,
    pub is_pinned: bool,
    pub is_closed: bool,
    pub unread_count: i32,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewTelegramTopic {
    pub topic_id: String,
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_topic_id: i64,
    pub provider_chat_id: String,
    pub title: String,
    pub icon_emoji: Option<String>,
    pub is_pinned: bool,
    pub is_closed: bool,
}

pub struct TelegramTopicListResponse {
    pub telegram_chat_id: String,
    pub items: Vec<TelegramTopic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramTopicCreateRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub title: String,
}

impl TelegramTopicCreateRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("title", &self.title)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramTopicCloseRequest {
    pub command_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub is_closed: bool,
}

impl TelegramTopicCloseRequest {
    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramTopicLifecycleResponse {
    pub operation: String,
    pub topic_id: Option<String>,
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_topic_id: Option<i64>,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub command_id: String,
}
