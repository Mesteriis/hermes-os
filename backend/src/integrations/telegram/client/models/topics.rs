use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

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
