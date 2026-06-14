use serde_json::Value;

use super::CallError;

pub(super) fn validate_limit(limit: i64) -> Result<i64, CallError> {
    if !(1..=100).contains(&limit) {
        return Err(CallError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<String, CallError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn validate_object(field: &'static str, value: &Value) -> Result<(), CallError> {
    if !value.is_object() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub(super) fn validate_array(field: &'static str, value: &Value) -> Result<(), CallError> {
    if !value.is_array() {
        return Err(CallError::InvalidRequest(format!(
            "{field} must be a JSON array"
        )));
    }
    Ok(())
}
