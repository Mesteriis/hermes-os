use crate::app::error::types::ApiError;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;

pub(super) fn required_string(field: &'static str, value: &str) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(WhatsappWebError::InvalidRequest(format!("{field} must not be empty")).into());
    }
    Ok(value.to_owned())
}

pub(super) fn optional_string(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, ApiError> {
    value
        .map(|value| required_string(field, &value))
        .transpose()
}

pub(super) fn parse_command_kinds(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
