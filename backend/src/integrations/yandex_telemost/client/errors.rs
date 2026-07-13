use hermes_communications_api::accounts::ProviderAccountPortError;
use hermes_communications_api::accounts::ProviderSecretBindingPortError;
use hermes_events_api::EventEnvelopeError;
use hermes_provider_telemost::protocol::YandexTelemostProtocolError;
use thiserror::Error;

use std::io;

use crate::domains::review::ReviewInboxError;

use crate::platform::secrets::{SecretReferenceError, SecretResolutionError};
use crate::platform::settings::SettingsError;
use crate::vault::HostVaultError;
use hermes_events_postgres::errors::EventStoreError;
use hermes_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum YandexTelemostError {
    #[error("invalid Yandex Telemost request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    ProviderAccountStore(#[from] ProviderAccountPortError),

    #[error(transparent)]
    ProviderSecretBindingStore(#[from] ProviderSecretBindingPortError),

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
    Io(#[from] io::Error),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Settings(#[from] SettingsError),
}

impl From<YandexTelemostProtocolError> for YandexTelemostError {
    fn from(error: YandexTelemostProtocolError) -> Self {
        match error {
            YandexTelemostProtocolError::InvalidRequest(message) => Self::InvalidRequest(message),
        }
    }
}
