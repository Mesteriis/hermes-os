use sqlx::postgres::PgPool;

use crate::domains::mail::messages::ProjectedMessage;

use super::errors::EmailRuleError;
use super::evaluation::{evaluate_conditions, parse_actions};
use super::mode::format_mode;
use super::models::{EmailRule, NewEmailRule, RuleMatchResult};
use super::rows::{EMAIL_RULE_COLUMNS, row_to_email_rule};

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
        let sql = format!(
            "INSERT INTO email_rules \
             (rule_id, name, description_nl, conditions_json, actions_json, mode, enabled) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (rule_id) DO UPDATE SET \
             name = EXCLUDED.name, \
             description_nl = EXCLUDED.description_nl, \
             conditions_json = EXCLUDED.conditions_json, \
             actions_json = EXCLUDED.actions_json, \
             mode = EXCLUDED.mode, \
             enabled = EXCLUDED.enabled, \
             updated_at = now() \
             RETURNING {EMAIL_RULE_COLUMNS}"
        );
        let row = sqlx::query(&sql)
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
        let sql = format!("SELECT {EMAIL_RULE_COLUMNS} FROM email_rules ORDER BY created_at DESC");
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
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
                results.push(RuleMatchResult {
                    rule_id: rule.rule_id.clone(),
                    matched: true,
                    matched_conditions,
                    suggested_actions: parse_actions(&rule.actions_json),
                });
            }
        }
        Ok(results)
    }
}
