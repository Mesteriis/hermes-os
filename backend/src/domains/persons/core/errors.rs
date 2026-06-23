use thiserror::Error;

use crate::platform::events::EventStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),

    #[error("person identity not found")]
    IdentityNotFound,

    #[error("person persona not found")]
    PersonaNotFound,
}
