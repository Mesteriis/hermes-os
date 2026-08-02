use crate::ContactsValidationErrorV1;

pub fn normalize_email_v1(value: &str) -> Result<String, ContactsValidationErrorV1> {
    let normalized = value.trim().to_lowercase();
    if normalized.is_empty()
        || normalized.len() > 320
        || normalized.chars().any(char::is_control)
        || normalized.matches('@').count() != 1
    {
        return Err(ContactsValidationErrorV1::InvalidEmail);
    }
    let (local, domain) = normalized
        .split_once('@')
        .ok_or(ContactsValidationErrorV1::InvalidEmail)?;
    if local.is_empty()
        || domain.is_empty()
        || domain.starts_with('.')
        || domain.ends_with('.')
        || !domain.contains('.')
        || local.chars().any(char::is_whitespace)
        || domain.chars().any(char::is_whitespace)
    {
        return Err(ContactsValidationErrorV1::InvalidEmail);
    }
    Ok(normalized)
}

pub fn normalize_phone_v1(value: &str) -> Result<String, ContactsValidationErrorV1> {
    let trimmed = value.trim();
    if !trimmed.starts_with('+') {
        return Err(ContactsValidationErrorV1::InvalidPhone);
    }
    let digits: String = trimmed
        .chars()
        .skip(1)
        .filter(|value| value.is_ascii_digit())
        .collect();
    if digits.len() < 7
        || digits.len() > 15
        || digits.starts_with('0')
        || trimmed
            .chars()
            .any(|value| !(value.is_ascii_digit() || matches!(value, '+' | ' ' | '-' | '(' | ')')))
    {
        return Err(ContactsValidationErrorV1::InvalidPhone);
    }
    Ok(format!("+{digits}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_normalization_is_stable_without_provider_rules() {
        assert_eq!(
            normalize_email_v1(" Ada@Example.COM ").expect("email"),
            "ada@example.com"
        );
        assert!(normalize_email_v1("ada").is_err());
    }

    #[test]
    fn phone_normalization_requires_explicit_international_identity() {
        assert_eq!(
            normalize_phone_v1("+34 (910) 000-000").expect("phone"),
            "+34910000000"
        );
        assert_eq!(
            normalize_phone_v1("910000000"),
            Err(ContactsValidationErrorV1::InvalidPhone)
        );
    }
}
