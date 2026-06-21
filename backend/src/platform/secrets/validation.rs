use serde_json::Value;

use super::database_vault::DatabaseEncryptedVaultError;
use super::errors::{SecretReferenceError, SecretResolutionError};
use super::file_vault::EncryptedVaultError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), SecretReferenceError> {
    if value.trim().is_empty() {
        return Err(SecretReferenceError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), SecretReferenceError> {
    if !value.is_object() {
        return Err(SecretReferenceError::NonObjectJson(field_name));
    }

    Ok(())
}

pub(super) fn validate_secret_resolution_ref(value: &str) -> Result<(), SecretResolutionError> {
    if value.trim().is_empty() {
        return Err(SecretResolutionError::EmptySecretRef);
    }

    Ok(())
}

pub(super) fn validate_vault_field(
    field: &'static str,
    value: &str,
) -> Result<(), EncryptedVaultError> {
    if value.trim().is_empty() {
        return Err(EncryptedVaultError::EmptyField(field));
    }
    Ok(())
}

pub(super) fn validate_database_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), DatabaseEncryptedVaultError> {
    if value.trim().is_empty() {
        return Err(DatabaseEncryptedVaultError::EmptyField(field));
    }

    Ok(())
}
