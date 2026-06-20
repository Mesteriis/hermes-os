use std::path::{Component, Path};

use super::constants::{LOCAL_FS_STORAGE_KIND, SHA256_PREFIX};
use super::errors::MailStorageError;

pub(crate) fn validate_storage_kind(value: &str) -> Result<String, MailStorageError> {
    let value = validate_non_empty("storage_kind", value)?;
    if value != LOCAL_FS_STORAGE_KIND {
        return Err(MailStorageError::InvalidStorageKind(value));
    }
    Ok(value)
}

pub(crate) fn validate_storage_path(value: &str) -> Result<String, MailStorageError> {
    let value = validate_non_empty("storage_path", value)?;
    let path = Path::new(value.as_str());
    if path.is_absolute() || value.contains('\\') {
        return Err(MailStorageError::UnsafeStoragePath(value));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(MailStorageError::UnsafeStoragePath(value)),
        }
    }

    Ok(value)
}

pub(crate) fn validate_sha256(value: &str) -> Result<String, MailStorageError> {
    let value = validate_non_empty("sha256", value)?;
    let Some(hex) = value.strip_prefix(SHA256_PREFIX) else {
        return Err(MailStorageError::InvalidSha256(value));
    };
    if hex.len() != 64 || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(MailStorageError::InvalidSha256(value));
    }
    Ok(format!("{SHA256_PREFIX}{}", hex.to_ascii_lowercase()))
}

pub(crate) fn validate_size_bytes(value: i64) -> Result<i64, MailStorageError> {
    if value < 0 {
        return Err(MailStorageError::NegativeSizeBytes(value));
    }
    Ok(value)
}

pub(crate) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<String, MailStorageError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(MailStorageError::EmptyField(field_name));
    }
    Ok(value)
}
