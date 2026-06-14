use thiserror::Error;

use crate::platform::secrets::SecretResolutionError;

#[derive(Debug, Error)]
pub enum HostVaultError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[cfg(target_os = "macos")]
    #[error(transparent)]
    Keyring(#[from] keyring::Error),

    #[error("host vault is not initialized")]
    Uninitialized,

    #[error("host vault is already initialized")]
    AlreadyInitialized,

    #[error("host vault is locked")]
    Locked,

    #[error("host vault state is poisoned")]
    StatePoisoned,

    #[error("insufficient vault entropy: collected {collected}, required {required}")]
    InsufficientEntropy { collected: usize, required: usize },

    #[error("entropy batch must not be empty")]
    EmptyEntropyBatch,

    #[error("host vault cryptographic operation failed")]
    Crypto,

    #[error("host vault random generation failed")]
    Random,

    #[error("invalid host vault encoding")]
    InvalidEncoding,

    #[error("invalid host vault recovery phrase")]
    InvalidRecoveryPhrase,

    #[error("unsupported host vault version: {0}")]
    UnsupportedVaultVersion(u16),

    #[error("secret was not found in host vault: {secret_ref}")]
    MissingSecret { secret_ref: String },

    #[error("host vault dev mode is forbidden in release builds")]
    DevModeForbiddenInRelease,

    #[error("host vault release runtime is macOS-only")]
    UnsupportedPlatform,

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl HostVaultError {
    fn public_message(&self) -> String {
        match self {
            Self::Crypto => "invalid host vault key or corrupted encrypted payload".to_owned(),
            Self::InvalidEncoding => "invalid host vault encoding".to_owned(),
            Self::InvalidRecoveryPhrase => "invalid host vault recovery phrase".to_owned(),
            Self::Locked => "host vault is locked".to_owned(),
            Self::Uninitialized => "host vault is not initialized".to_owned(),
            Self::MissingSecret { secret_ref } => format!("secret was not found: {secret_ref}"),
            Self::EmptyField(field) => format!("{field} must not be empty"),
            Self::InsufficientEntropy {
                collected,
                required,
            } => {
                format!("insufficient entropy: collected {collected}, required {required}")
            }
            Self::AlreadyInitialized => "host vault is already initialized".to_owned(),
            Self::EmptyEntropyBatch => "entropy batch must not be empty".to_owned(),
            Self::UnsupportedVaultVersion(_) => "unsupported host vault version".to_owned(),
            Self::DevModeForbiddenInRelease => {
                "host vault dev mode is forbidden in release".to_owned()
            }
            Self::UnsupportedPlatform => "host vault release runtime is macOS-only".to_owned(),
            Self::Io(_) | Self::Sqlite(_) | Self::Json(_) | Self::StatePoisoned | Self::Random => {
                "host vault operation failed".to_owned()
            }
            #[cfg(target_os = "macos")]
            Self::Keyring(_) => "macOS Keychain operation failed".to_owned(),
        }
    }
}

pub(super) fn host_secret_store_failure(error: HostVaultError) -> SecretResolutionError {
    SecretResolutionError::StoreFailure {
        message: error.public_message(),
    }
}
