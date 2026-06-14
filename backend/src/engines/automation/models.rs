use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewAutomationTemplate {
    pub template_id: String,
    pub name: String,
    pub body_template: String,
    pub required_variables: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AutomationTemplate {
    pub template_id: String,
    pub name: String,
    pub body_template: String,
    pub required_variables: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewAutomationPolicy {
    pub policy_id: String,
    pub template_id: String,
    pub name: String,
    pub enabled: bool,
    pub account_id: String,
    pub allowed_chat_ids: Vec<String>,
    pub trigger_kind: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AutomationPolicy {
    pub policy_id: String,
    pub template_id: String,
    pub name: String,
    pub enabled: bool,
    pub account_id: String,
    pub allowed_chat_ids: Vec<String>,
    pub trigger_kind: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramSendDryRunRequest {
    pub command_id: String,
    pub policy_id: String,
    pub provider_chat_id: String,
    #[serde(default)]
    pub variables: Value,
    #[serde(default)]
    pub source_context: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramSendDryRunResponse {
    pub outbound_message_id: String,
    pub policy_id: String,
    pub template_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub rendered_text: String,
    pub rendered_preview_hash: String,
    pub status: String,
    pub event_id: String,
}

pub fn object_from_pairs(pairs: impl IntoIterator<Item = (String, Value)>) -> Value {
    Value::Object(Map::from_iter(pairs))
}
