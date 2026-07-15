use super::constants::{DEFAULT_LIMIT, MAX_LIMIT, MIN_LIMIT};
use super::errors::TaskCandidateError;

pub(crate) fn validate_non_empty(field: &str, value: &str) -> Result<String, TaskCandidateError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TaskCandidateError::EmptyField(field.to_owned()));
    }

    Ok(value.to_owned())
}

pub(crate) fn validate_limit(limit: i64) -> Result<i64, TaskCandidateError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(TaskCandidateError::InvalidLimit);
    }

    Ok(limit)
}

pub(crate) fn validate_optional_limit(limit: Option<i64>) -> Result<i64, TaskCandidateError> {
    validate_limit(limit.unwrap_or(DEFAULT_LIMIT))
}

pub(crate) fn text_preview(value: &str, max_chars: usize) -> String {
    let preview = value.trim().chars().take(max_chars).collect::<String>();
    if value.trim().chars().count() > max_chars {
        format!("{preview}...")
    } else {
        preview
    }
}
