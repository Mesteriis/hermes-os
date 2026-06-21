use serde_json::Value;

use super::errors::TelegramError;

pub(super) fn validate_message_list_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=5000).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "message list limit must be between 1 and 5000".to_owned(),
        ));
    }
    Ok(limit)
}

pub(super) fn validate_chat_list_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=5000).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "chat list limit must be between 1 and 5000".to_owned(),
        ));
    }
    Ok(limit)
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, TelegramError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn required_optional_value(
    field: &'static str,
    value: Option<&str>,
) -> Result<String, TelegramError> {
    let Some(value) = value else {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    };

    validate_non_empty(field, value)
}

pub(super) fn validate_object(field: &'static str, value: &Value) -> Result<(), TelegramError> {
    if !value.is_object() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}
