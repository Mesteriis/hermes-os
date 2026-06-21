use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::mode::RuleMode;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailRule {
    pub rule_id: String,
    pub name: String,
    pub description_nl: String,
    pub conditions_json: Value,
    pub actions_json: Value,
    pub mode: RuleMode,
    pub enabled: bool,
    pub match_count: i64,
    pub last_matched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMatchResult {
    pub rule_id: String,
    pub matched: bool,
    pub matched_conditions: Vec<String>,
    pub suggested_actions: Vec<RuleAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: String,
    pub params: Value,
}

#[derive(Clone, Debug)]
pub struct NewEmailRule {
    pub rule_id: String,
    pub name: String,
    pub description_nl: String,
    pub conditions_json: Value,
    pub actions_json: Value,
    pub mode: RuleMode,
    pub enabled: bool,
}
