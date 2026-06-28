use thiserror::Error;

use crate::platform::events::EventStoreError;
use crate::platform::settings::SettingsError;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to connect to PostgreSQL")]
    Connect(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Settings(#[from] SettingsError),

    #[error("{0}")]
    Invalid(String),
}
