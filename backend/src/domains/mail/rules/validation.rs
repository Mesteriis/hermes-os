use super::errors::EmailRuleError;
use super::models::NewEmailRule;

impl NewEmailRule {
    pub(in crate::domains::mail::rules) fn validate(&self) -> Result<(), EmailRuleError> {
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
