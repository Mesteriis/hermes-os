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
    /// Deprecated Telegram compatibility input. It is materialized as
    /// `telegram.chat` scopes before the policy is persisted.
    pub allowed_chat_ids: Vec<String>,
    pub scopes: Vec<AutomationPolicyScope>,
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
    /// Deprecated Telegram compatibility projection.
    pub allowed_chat_ids: Vec<String>,
    pub scopes: Vec<AutomationPolicyScope>,
    pub trigger_kind: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationPolicyScope {
    pub scope_kind: String,
    pub scope_value: String,
}

impl AutomationPolicy {
    pub fn allows_scope(&self, scope_kind: &str, scope_value: &str) -> bool {
        self.scopes
            .iter()
            .any(|scope| scope.scope_kind == scope_kind && scope.scope_value == scope_value.trim())
    }
}

impl NewAutomationPolicy {
    pub(super) fn normalized_scopes(&self) -> Vec<AutomationPolicyScope> {
        let mut scopes = self
            .scopes
            .iter()
            .map(|scope| AutomationPolicyScope {
                scope_kind: scope.scope_kind.trim().to_owned(),
                scope_value: scope.scope_value.trim().to_owned(),
            })
            .collect::<Vec<_>>();
        scopes.extend(
            self.allowed_chat_ids
                .iter()
                .map(|scope_value| AutomationPolicyScope {
                    scope_kind: "telegram.chat".to_owned(),
                    scope_value: scope_value.trim().to_owned(),
                }),
        );
        scopes.sort_by(|left, right| {
            left.scope_kind
                .cmp(&right.scope_kind)
                .then(left.scope_value.cmp(&right.scope_value))
        });
        scopes.dedup();
        scopes
    }
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
