use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramMessageReferenceSummary {
    pub message_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: Option<String>,
    pub chat_title: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub text: String,
    pub occurred_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramReplyRef {
    pub reply_ref_id: String,
    pub source_message_id: String,
    pub target_message_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub source_provider_id: String,
    pub target_provider_id: String,
    pub reply_depth: i32,
    pub is_topic_reply: bool,
    pub topic_id: Option<String>,
    pub source_message_summary: Option<TelegramMessageReferenceSummary>,
    pub target_message_summary: Option<TelegramMessageReferenceSummary>,
    pub metadata: serde_json::Value,
    pub provenance: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TelegramForwardRef {
    pub forward_ref_id: String,
    pub source_message_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub source_provider_id: String,
    pub forward_origin_chat_id: Option<String>,
    pub forward_origin_message_id: Option<String>,
    pub forward_origin_sender_id: Option<String>,
    pub forward_origin_sender_name: Option<String>,
    pub forward_date: Option<DateTime<Utc>>,
    pub forward_depth: i32,
    pub source_message_summary: Option<TelegramMessageReferenceSummary>,
    pub metadata: serde_json::Value,
    pub provenance: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramReplyChainResponse {
    pub message_id: String,
    pub replies: Vec<TelegramReplyRef>,
    pub reply_to: Vec<TelegramReplyRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramForwardChainResponse {
    pub message_id: String,
    pub forwards: Vec<TelegramForwardRef>,
}
