use super::errors::ProjectLinkReviewError;

pub(crate) fn validate_non_empty(
    field: &str,
    value: &str,
) -> Result<String, ProjectLinkReviewError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ProjectLinkReviewError::EmptyField(field.to_owned()));
    }

    Ok(normalized.to_owned())
}
