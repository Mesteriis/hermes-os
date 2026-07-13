use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum IdentifierError {
    #[error("{kind} must not be empty")]
    Empty { kind: &'static str },
    #[error("{kind} must use lowercase ASCII letters, digits, `_` or `-`")]
    InvalidCharacters { kind: &'static str },
}

pub fn validate_slug(value: &str, kind: &'static str) -> Result<(), IdentifierError> {
    if value.trim().is_empty() {
        return Err(IdentifierError::Empty { kind });
    }

    if !value.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
    }) {
        return Err(IdentifierError::InvalidCharacters { kind });
    }

    Ok(())
}
