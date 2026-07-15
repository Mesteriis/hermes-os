use super::constants::{DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT, MIN_LIST_LIMIT};
use super::errors::DocumentProcessingError;

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, DocumentProcessingError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(DocumentProcessingError::EmptyField(field));
    }

    Ok(value.to_owned())
}

pub(super) fn validate_limit(limit: i64) -> Result<i64, DocumentProcessingError> {
    if !(MIN_LIST_LIMIT..=MAX_LIST_LIMIT).contains(&limit) {
        return Err(DocumentProcessingError::InvalidLimit);
    }
    Ok(limit)
}

pub(super) fn validate_optional_limit(limit: Option<i64>) -> Result<i64, DocumentProcessingError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIST_LIMIT))
}
