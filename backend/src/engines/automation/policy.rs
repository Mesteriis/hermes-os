use chrono::Utc;
use serde_json::Value;

use super::errors::AutomationError;
use super::models::{AutomationPolicy, AutomationTemplate, TelegramSendDryRunRequest};

pub(super) fn evaluate_policy(
    policy: &AutomationPolicy,
    template: &AutomationTemplate,
    request: &TelegramSendDryRunRequest,
) -> Result<String, AutomationError> {
    if !policy.enabled {
        return Err(AutomationError::PolicyDisabled);
    }
    if let Some(expires_at) = policy.expires_at
        && expires_at < Utc::now()
    {
        return Err(AutomationError::InvalidRequest(
            "policy is expired".to_owned(),
        ));
    }
    if !policy
        .allowed_chat_ids
        .iter()
        .any(|chat_id| chat_id == &request.provider_chat_id)
    {
        return Err(AutomationError::ChatNotAllowed);
    }
    let variables = request
        .variables
        .as_object()
        .ok_or_else(|| AutomationError::InvalidRequest("variables must be an object".to_owned()))?;
    for key in variables.keys() {
        if !template
            .required_variables
            .iter()
            .any(|allowed| allowed == key)
        {
            return Err(AutomationError::UndeclaredTemplateVariable(key.clone()));
        }
    }

    let mut rendered = template.body_template.clone();
    for variable in &template.required_variables {
        let value = variables
            .get(variable)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| AutomationError::MissingTemplateVariable(variable.clone()))?;
        rendered = rendered.replace(&format!("{{{{{variable}}}}}"), value);
    }

    Ok(rendered)
}
