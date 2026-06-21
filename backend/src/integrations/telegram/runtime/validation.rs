use crate::integrations::telegram::client::TelegramError;

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

pub(super) fn validate_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=100).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}
