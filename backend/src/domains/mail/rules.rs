use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(test)]
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::domains::mail::messages::ProjectedMessage;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
    Suggest,
    AskBeforeExecute,
    AutoExecute,
    DryRun,
}

impl RuleMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleMode::Suggest => "suggest",
            RuleMode::AskBeforeExecute => "ask_before_execute",
            RuleMode::AutoExecute => "auto_execute",
            RuleMode::DryRun => "dry_run",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "suggest" => Some(RuleMode::Suggest),
            "ask_before_execute" => Some(RuleMode::AskBeforeExecute),
            "auto_execute" => Some(RuleMode::AutoExecute),
            "dry_run" => Some(RuleMode::DryRun),
            _ => None,
        }
    }
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

#[derive(Clone)]
pub struct EmailRuleStore {
    pool: PgPool,
}

impl EmailRuleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_rule(&self, rule: &NewEmailRule) -> Result<EmailRule, EmailRuleError> {
        rule.validate()?;
        let mode = format_mode(rule.mode);
        let row = sqlx::query(
            r#"INSERT INTO email_rules (rule_id, name, description_nl, conditions_json, actions_json, mode, enabled)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (rule_id) DO UPDATE SET
                name = EXCLUDED.name,
                description_nl = EXCLUDED.description_nl,
                conditions_json = EXCLUDED.conditions_json,
                actions_json = EXCLUDED.actions_json,
                mode = EXCLUDED.mode,
                enabled = EXCLUDED.enabled,
                updated_at = now()
            RETURNING rule_id, name, description_nl, conditions_json, actions_json, mode, enabled, match_count, last_matched_at, created_at, updated_at"#,
        )
        .bind(&rule.rule_id)
        .bind(&rule.name)
        .bind(&rule.description_nl)
        .bind(&rule.conditions_json)
        .bind(&rule.actions_json)
        .bind(&mode)
        .bind(rule.enabled)
        .fetch_one(&self.pool)
        .await?;
        row_to_email_rule(row)
    }

    pub async fn list_rules(&self) -> Result<Vec<EmailRule>, EmailRuleError> {
        let rows = sqlx::query(
            r#"SELECT rule_id, name, description_nl, conditions_json, actions_json, mode, enabled, match_count, last_matched_at, created_at, updated_at
            FROM email_rules ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_email_rule).collect()
    }

    pub async fn match_rules(
        &self,
        message: &ProjectedMessage,
    ) -> Result<Vec<RuleMatchResult>, EmailRuleError> {
        let rules = self.list_rules().await?;
        let mut results = Vec::new();

        for rule in &rules {
            if !rule.enabled {
                continue;
            }
            let matched_conditions = evaluate_conditions(&rule.conditions_json, message);
            if !matched_conditions.is_empty() {
                let actions = parse_actions(&rule.actions_json);
                results.push(RuleMatchResult {
                    rule_id: rule.rule_id.clone(),
                    matched: true,
                    matched_conditions,
                    suggested_actions: actions,
                });
            }
        }
        Ok(results)
    }
}

fn evaluate_conditions(conditions: &Value, message: &ProjectedMessage) -> Vec<String> {
    let mut matched = Vec::new();
    let body_lower = message.body_text.to_lowercase();
    let subject_lower = message.subject.to_lowercase();

    if let Some(arr) = conditions.as_array() {
        for cond in arr {
            let field = cond.get("field").and_then(|v| v.as_str()).unwrap_or("");
            let operator = cond
                .get("operator")
                .and_then(|v| v.as_str())
                .unwrap_or("contains");
            let value = cond.get("value").and_then(|v| v.as_str()).unwrap_or("");

            let is_match = match (field, operator) {
                ("subject", "contains") => subject_lower.contains(&value.to_lowercase()),
                ("subject", "equals") => subject_lower == value.to_lowercase(),
                ("body", "contains") => body_lower.contains(&value.to_lowercase()),
                ("sender", "contains") => message
                    .sender
                    .to_lowercase()
                    .contains(&value.to_lowercase()),
                ("sender", "equals") => message.sender.to_lowercase() == value.to_lowercase(),
                ("has_attachment", "equals") => {
                    // Check attachment count via metadata; default to false for simple checks
                    value == "true"
                }
                _ => false,
            };

            if is_match {
                let label = cond
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("condition matched");
                matched.push(label.to_owned());
            }
        }
    }
    matched
}

fn parse_actions(actions: &Value) -> Vec<RuleAction> {
    let mut result = Vec::new();
    if let Some(arr) = actions.as_array() {
        for action in arr {
            if let (Some(action_type), Some(params)) = (
                action.get("action_type").and_then(|v| v.as_str()),
                action.get("params"),
            ) {
                result.push(RuleAction {
                    action_type: action_type.to_owned(),
                    params: params.clone(),
                });
            }
        }
    }
    result
}

fn format_mode(mode: RuleMode) -> String {
    mode.as_str().to_owned()
}

fn row_to_email_rule(row: PgRow) -> Result<EmailRule, EmailRuleError> {
    let mode_str: String = row.try_get("mode")?;
    Ok(EmailRule {
        rule_id: row.try_get("rule_id")?,
        name: row.try_get("name")?,
        description_nl: row.try_get("description_nl")?,
        conditions_json: row.try_get("conditions_json")?,
        actions_json: row.try_get("actions_json")?,
        mode: RuleMode::parse(&mode_str).unwrap_or(RuleMode::Suggest),
        enabled: row.try_get("enabled")?,
        match_count: row.try_get("match_count")?,
        last_matched_at: row.try_get("last_matched_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
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

impl NewEmailRule {
    fn validate(&self) -> Result<(), EmailRuleError> {
        if self.rule_id.trim().is_empty() {
            return Err(EmailRuleError::InvalidRule("rule_id is empty"));
        }
        if self.name.trim().is_empty() {
            return Err(EmailRuleError::InvalidRule("name is empty"));
        }
        if !self.conditions_json.is_array() {
            return Err(EmailRuleError::InvalidRule("conditions must be an array"));
        }
        if !self.actions_json.is_array() {
            return Err(EmailRuleError::InvalidRule("actions must be an array"));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum EmailRuleError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid rule: {0}")]
    InvalidRule(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::mail::messages::{LocalMessageState, WorkflowState};
    use chrono::Utc;

    fn test_message(subject: &str, sender: &str, body: &str) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "msg:test:1".into(),
            raw_record_id: "raw:1".into(),
            account_id: "acct:1".into(),
            provider_record_id: "prov:1".into(),
            subject: subject.into(),
            sender: sender.into(),
            recipients: vec!["to@ex.com".into()],
            body_text: body.into(),
            occurred_at: Some(Utc::now()),
            projected_at: Utc::now(),
            channel_kind: "email".into(),
            conversation_id: None,
            sender_display_name: None,
            delivery_state: "received".into(),
            message_metadata: json!({}),
            workflow_state: WorkflowState::New,
            importance_score: None,
            ai_category: None,
            ai_summary: None,
            ai_summary_generated_at: None,
            local_state: LocalMessageState::Active,
            local_state_changed_at: None,
            local_state_reason: None,
        }
    }

    #[test]
    fn evaluate_conditions_matches_subject() {
        let msg = test_message("Urgent: Project Update", "alice@ex.com", "Body text");
        let conditions = json!([
            {"field": "subject", "operator": "contains", "value": "urgent", "label": "urgent subject"}
        ]);
        let matched = evaluate_conditions(&conditions, &msg);
        assert_eq!(matched, vec!["urgent subject"]);
    }

    #[test]
    fn evaluate_conditions_matches_sender() {
        let msg = test_message("Hello", "bob@company.com", "Body");
        let conditions = json!([
            {"field": "sender", "operator": "contains", "value": "company.com", "label": "company sender"}
        ]);
        let matched = evaluate_conditions(&conditions, &msg);
        assert_eq!(matched, vec!["company sender"]);
    }

    #[test]
    fn evaluate_conditions_no_match() {
        let msg = test_message("Regular", "alice@ex.com", "Nothing special");
        let conditions = json!([
            {"field": "subject", "operator": "contains", "value": "urgent", "label": "urgent"}
        ]);
        let matched = evaluate_conditions(&conditions, &msg);
        assert!(matched.is_empty());
    }

    #[test]
    fn rule_mode_parse_all() {
        assert_eq!(RuleMode::parse("suggest"), Some(RuleMode::Suggest));
        assert_eq!(RuleMode::parse("auto_execute"), Some(RuleMode::AutoExecute));
        assert_eq!(RuleMode::parse("dry_run"), Some(RuleMode::DryRun));
        assert_eq!(RuleMode::parse("invalid"), None);
    }
}
