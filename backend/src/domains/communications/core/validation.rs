use serde_json::Value;

use super::errors::CommunicationIngestionError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), CommunicationIngestionError> {
    if value.trim().is_empty() {
        return Err(CommunicationIngestionError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), CommunicationIngestionError> {
    if !value.is_object() {
        return Err(CommunicationIngestionError::NonObjectJson(field_name));
    }

    Ok(())
}
