use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZoomProtocolError {
    #[error("invalid Zoom request: {0}")]
    InvalidRequest(String),
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<String, ZoomProtocolError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ZoomProtocolError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub fn validate_object(field: &'static str, value: &Value) -> Result<(), ZoomProtocolError> {
    if !value.is_object() {
        return Err(ZoomProtocolError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub fn validate_array(field: &'static str, value: &Value) -> Result<(), ZoomProtocolError> {
    if !value.is_array() {
        return Err(ZoomProtocolError::InvalidRequest(format!(
            "{field} must be a JSON array"
        )));
    }
    Ok(())
}
