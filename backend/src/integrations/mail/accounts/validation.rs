use super::errors::EmailAccountSetupError;

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), EmailAccountSetupError> {
    if value.trim().is_empty() {
        return Err(EmailAccountSetupError::InvalidRequest {
            field,
            message: "must not be empty",
        });
    }

    Ok(())
}
