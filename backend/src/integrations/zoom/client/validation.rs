use serde_json::Value;

use super::errors::ZoomError;

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<String, ZoomError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn validate_object(field: &'static str, value: &Value) -> Result<(), ZoomError> {
    if !value.is_object() {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub(super) fn validate_array(field: &'static str, value: &Value) -> Result<(), ZoomError> {
    if !value.is_array() {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must be a JSON array"
        )));
    }
    Ok(())
}
