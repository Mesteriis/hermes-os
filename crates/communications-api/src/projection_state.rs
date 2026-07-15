use chrono::{DateTime, Utc};
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Debug, PartialEq)]
pub struct MessageProjectionState {
    pub message_id: String,
    pub workflow_state: String,
    pub local_state: String,
    pub local_state_changed_at: Option<DateTime<Utc>>,
    pub local_state_reason: Option<String>,
    pub importance_score: Option<i16>,
    pub ai_category: Option<String>,
    pub ai_summary: Option<String>,
    pub ai_summary_generated_at: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub read_changed_at: Option<DateTime<Utc>>,
    pub read_origin: String,
}

pub type MessageProjectionStateFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<Option<MessageProjectionState>, MessageProjectionStateError>>
            + Send
            + 'a,
    >,
>;

pub trait MessageProjectionStateQueryPort: Send + Sync {
    fn state<'a>(&'a self, message_id: &'a str) -> MessageProjectionStateFuture<'a>;
}

#[derive(Debug, thiserror::Error)]
#[error("message projection state query failed: {0}")]
pub struct MessageProjectionStateError(pub String);
