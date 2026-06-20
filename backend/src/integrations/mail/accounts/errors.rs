use thiserror::Error;

use crate::platform::secrets::{
    DatabaseEncryptedVaultError, SecretReferenceError, SecretResolutionError,
};
use crate::vault::HostVaultError;

#[derive(Debug, Error)]
pub enum EmailAccountSetupError {
    #[error("invalid account setup request field {field}: {message}")]
    InvalidRequest {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider response is missing required field: {field}")]
    MissingProviderField { field: &'static str },

    #[error("account setup stores are not configured")]
    StoresNotConfigured,

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    DatabaseVault(#[from] DatabaseEncryptedVaultError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    Secret(#[from] SecretResolutionError),

    #[error("provider account store operation failed: {0}")]
    ProviderAccountStore(String),
}
