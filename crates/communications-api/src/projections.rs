use chrono::{DateTime, Utc};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Debug, PartialEq)]
pub struct MessageProjectionInput {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub channel_kind: String,
    pub conversation_id: Option<String>,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MessageProjectionRead {
    pub message_id: String,
    pub raw_record_id: String,
    pub observation_id: String,
    pub account_id: String,
    pub provider_record_id: String,
    pub subject: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub body_text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub conversation_id: Option<String>,
    pub sender_display_name: Option<String>,
    pub delivery_state: String,
    pub metadata: Value,
}

pub type MessageProjectionWriteFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<MessageProjectionRead, MessageProjectionWriteError>> + Send + 'a,
    >,
>;

pub trait MessageProjectionWritePort: Send + Sync {
    fn upsert<'a>(&'a self, input: &'a MessageProjectionInput) -> MessageProjectionWriteFuture<'a>;
}

#[derive(Debug, thiserror::Error)]
pub enum MessageProjectionWriteError {
    #[error("invalid message projection: {0}")]
    InvalidInput(&'static str),
    #[error("message projection write failed: {0}")]
    Failed(String),
}
