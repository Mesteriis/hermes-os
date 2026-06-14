use serde_json::Value;

use crate::domains::mail::messages::ProjectedMessage;

use super::models::RuleAction;

pub(in crate::domains::mail::rules) fn evaluate_conditions(
    conditions: &Value,
    message: &ProjectedMessage,
) -> Vec<String> {
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
                ("has_attachment", "equals") => value == "true",
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

pub(in crate::domains::mail::rules) fn parse_actions(actions: &Value) -> Vec<RuleAction> {
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
