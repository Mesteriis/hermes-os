use crate::errors::CommunicationIngestionError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), CommunicationIngestionError> {
    if value.trim().is_empty() {
        return Err(CommunicationIngestionError::EmptyField(field_name));
    }

    Ok(())
}
