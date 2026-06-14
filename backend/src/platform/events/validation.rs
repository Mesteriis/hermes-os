use serde_json::Value;

use super::errors::EventEnvelopeError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), EventEnvelopeError> {
    if value.trim().is_empty() {
        return Err(EventEnvelopeError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), EventEnvelopeError> {
    if !value.is_object() {
        return Err(EventEnvelopeError::NonObjectJson(field_name));
    }

    Ok(())
}
