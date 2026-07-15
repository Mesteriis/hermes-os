use super::WhatsappWebError;

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<String, WhatsappWebError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::validate_non_empty;

    #[test]
    fn rejects_empty_and_whitespace_values() {
        assert!(validate_non_empty("field", "").is_err());
        assert!(validate_non_empty("field", "   ").is_err());
    }

    #[test]
    fn trims_valid_values() {
        assert_eq!(validate_non_empty("field", " value ").unwrap(), "value");
    }
}
