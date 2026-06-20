use super::errors::MessageProjectionError;

pub(crate) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), MessageProjectionError> {
    if value.trim().is_empty() {
        return Err(MessageProjectionError::EmptyField(field_name));
    }

    Ok(())
}

pub(crate) fn validate_limit(limit: i64) -> Result<i64, MessageProjectionError> {
    if !(1..=5000).contains(&limit) {
        return Err(MessageProjectionError::InvalidLimit(limit));
    }

    Ok(limit)
}
