use hermes_communications_api::accounts::ProviderAccountPortError;
use hermes_communications_api::accounts::ProviderSecretBindingPortError;
use hermes_events_api::EventEnvelopeError;
use hermes_provider_zoom::protocol::ZoomProtocolError;
use thiserror::Error;

use crate::platform::calls::errors::CallError;

use crate::platform::secrets::{SecretReferenceError, SecretResolutionError};
use crate::platform::settings::SettingsError;
use crate::platform::storage::StorageError;
use crate::vault::HostVaultError;
use hermes_events_postgres::errors::EventStoreError;

#[derive(Debug, Error)]
pub enum ZoomError {
    #[error("invalid Zoom request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    ProviderAccountStore(#[from] ProviderAccountPortError),

    #[error(transparent)]
    ProviderSecretBindingStore(#[from] ProviderSecretBindingPortError),

    #[error(transparent)]
    Call(#[from] CallError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Settings(#[from] SettingsError),
}

impl From<ZoomProtocolError> for ZoomError {
    fn from(error: ZoomProtocolError) -> Self {
        match error {
            ZoomProtocolError::InvalidRequest(message) => Self::InvalidRequest(message),
        }
    }
}
