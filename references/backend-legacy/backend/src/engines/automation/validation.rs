use serde_json::Value;

use super::errors::AutomationError;
use super::models::{
    AutomationPolicyScope, NewAutomationPolicy, NewAutomationTemplate, TelegramSendDryRunRequest,
};

impl NewAutomationTemplate {
    pub(super) fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("template_id", &self.template_id)?;
        validate_non_empty("name", &self.name)?;
        validate_non_empty("body_template", &self.body_template)?;
        for variable in &self.required_variables {
            validate_variable_name(variable)?;
        }
        Ok(())
    }
}

impl NewAutomationPolicy {
    pub(super) fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("policy_id", &self.policy_id)?;
        validate_non_empty("template_id", &self.template_id)?;
        validate_non_empty("name", &self.name)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("trigger_kind", &self.trigger_kind)?;
        if self.max_sends_per_hour <= 0 {
            return Err(AutomationError::InvalidRequest(
                "max_sends_per_hour must be greater than zero".to_owned(),
            ));
        }
        let scopes = self.normalized_scopes();
        if scopes.is_empty() {
            return Err(AutomationError::InvalidRequest(
                "automation policy must include at least one scope".to_owned(),
            ));
        }
        for scope in &scopes {
            validate_scope(scope)?;
        }
        validate_object("quiet_hours", &self.quiet_hours)?;
        validate_object("conditions", &self.conditions)?;
        Ok(())
    }
}

fn validate_scope(scope: &AutomationPolicyScope) -> Result<(), AutomationError> {
    let kind = validate_non_empty("scope_kind", &scope.scope_kind)?;
    if kind.len() > 80
        || !kind.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-')
        })
    {
        return Err(AutomationError::InvalidRequest(
            "scope_kind must use lowercase ASCII letters, numbers, '.', '_' or '-'".to_owned(),
        ));
    }
    let value = validate_non_empty("scope_value", &scope.scope_value)?;
    if value.len() > 512 {
        return Err(AutomationError::InvalidRequest(
            "scope_value must be at most 512 bytes".to_owned(),
        ));
    }
    Ok(())
}

impl TelegramSendDryRunRequest {
    pub(super) fn validate(&self) -> Result<(), AutomationError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("policy_id", &self.policy_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_object("variables", &self.variables)?;
        validate_object("source_context", &self.source_context)?;
        Ok(())
    }
}

pub(super) fn validate_variable_name(value: &str) -> Result<String, AutomationError> {
    let value = validate_non_empty("required_variable", value)?;
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(AutomationError::InvalidRequest(
            "template variables must be ASCII letters, numbers or underscores".to_owned(),
        ));
    }
    Ok(value)
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, AutomationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AutomationError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), AutomationError> {
    if !matches!(value, Value::Object(_)) {
        return Err(AutomationError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}
