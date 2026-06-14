use serde_json::Value;

use super::errors::WhatsappWebError;

pub(crate) fn validate_limit(limit: i64) -> Result<i64, WhatsappWebError> {
    if !(1..=100).contains(&limit) {
        return Err(WhatsappWebError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

pub(crate) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, WhatsappWebError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn validate_object(field: &'static str, value: &Value) -> Result<(), WhatsappWebError> {
    if !value.is_object() {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}
