use super::constants::MAX_PROJECT_LIMIT;
use super::errors::ProjectStoreError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<String, ProjectStoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProjectStoreError::EmptyField(field_name));
    }

    Ok(trimmed.to_owned())
}

pub(super) fn validate_limit(limit: i64) -> Result<i64, ProjectStoreError> {
    if limit <= 0 {
        return Err(ProjectStoreError::InvalidLimit);
    }

    Ok(limit.min(MAX_PROJECT_LIMIT))
}
