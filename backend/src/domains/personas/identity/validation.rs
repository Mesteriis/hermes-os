use serde_json::Value;

use super::constants::{DEFAULT_LIMIT, MAX_LIMIT, MIN_LIMIT};
use super::errors::PersonaIdentityError;

pub(super) fn as_object(
    value: &Value,
) -> Result<&serde_json::Map<String, Value>, PersonaIdentityError> {
    value
        .as_object()
        .ok_or_else(|| PersonaIdentityError::InvalidPayload("payload".to_owned()))
}

pub(super) fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, PersonaIdentityError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| PersonaIdentityError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| PersonaIdentityError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}

pub(super) fn validate_non_empty(field: &str, value: &str) -> Result<String, PersonaIdentityError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(PersonaIdentityError::EmptyField(field.to_owned()));
    }

    Ok(normalized.to_owned())
}

pub(super) fn validate_limit(limit: i64) -> Result<i64, PersonaIdentityError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(PersonaIdentityError::InvalidLimit);
    }

    Ok(limit)
}

pub(super) fn validate_optional_limit(limit: Option<i64>) -> Result<i64, PersonaIdentityError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIMIT))
}
