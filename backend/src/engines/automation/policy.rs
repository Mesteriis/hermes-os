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
    if !policy.allows_scope("telegram.chat", &request.provider_chat_id) {
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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::engines::automation::models::AutomationPolicyScope;

    fn policy(scopes: Vec<AutomationPolicyScope>) -> AutomationPolicy {
        AutomationPolicy {
            policy_id: "policy-1".to_owned(),
            template_id: "template-1".to_owned(),
            name: "Test policy".to_owned(),
            enabled: true,
            account_id: "account-1".to_owned(),
            allowed_chat_ids: vec!["legacy-chat".to_owned()],
            scopes,
            trigger_kind: "test".to_owned(),
            max_sends_per_hour: 1,
            quiet_hours: json!({}),
            expires_at: None,
            conditions: json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn template() -> AutomationTemplate {
        AutomationTemplate {
            template_id: "template-1".to_owned(),
            name: "Test template".to_owned(),
            body_template: "Hi {{name}}".to_owned(),
            required_variables: vec!["name".to_owned()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn request(chat_id: &str) -> TelegramSendDryRunRequest {
        TelegramSendDryRunRequest {
            command_id: "command-1".to_owned(),
            policy_id: "policy-1".to_owned(),
            provider_chat_id: chat_id.to_owned(),
            variables: json!({ "name": "Ada" }),
            source_context: json!({}),
        }
    }

    #[test]
    fn telegram_send_requires_a_durable_telegram_scope() {
        let result = evaluate_policy(&policy(Vec::new()), &template(), &request("legacy-chat"));

        assert!(matches!(result, Err(AutomationError::ChatNotAllowed)));
    }

    #[test]
    fn telegram_send_accepts_the_matching_generic_scope() {
        let result = evaluate_policy(
            &policy(vec![AutomationPolicyScope {
                scope_kind: "telegram.chat".to_owned(),
                scope_value: "chat-1".to_owned(),
            }]),
            &template(),
            &request("chat-1"),
        );

        assert_eq!(result.expect("allowed by scope"), "Hi Ada");
    }
}
