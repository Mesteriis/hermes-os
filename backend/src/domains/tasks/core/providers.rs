use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskProviderAccount {
    pub account_id: String,
    pub provider: String,
    pub account_name: String,
    pub credentials_reference: Option<String>,
    pub sync_mode: String,
    pub capabilities: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
