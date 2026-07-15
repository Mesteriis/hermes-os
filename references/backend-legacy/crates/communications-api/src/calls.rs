use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct CanonicalCallRecord {
    pub call_id: String,
    pub account_id: String,
    pub provider_call_id: String,
    pub provider_chat_id: String,
    pub direction: String,
    pub call_state: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

#[derive(Debug, thiserror::Error)]
#[error("canonical call read failed: {0}")]
pub struct CanonicalCallReadError(pub String);

#[async_trait::async_trait]
pub trait CanonicalCallReadPort: Send + Sync {
    async fn list_whatsapp_calls(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CanonicalCallRecord>, CanonicalCallReadError>;
}
