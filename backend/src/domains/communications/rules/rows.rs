use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::EmailRuleError;
use super::mode::RuleMode;
use super::models::EmailRule;

pub(in crate::domains::communications::rules) const EMAIL_RULE_COLUMNS: &str = "rule_id, name, description_nl, conditions_json, actions_json, mode, enabled, match_count, \
     last_matched_at, created_at, updated_at";

pub(in crate::domains::communications::rules) fn row_to_email_rule(
    row: PgRow,
) -> Result<EmailRule, EmailRuleError> {
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
