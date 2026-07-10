use thiserror::Error;

use crate::platform::events::EventStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonaCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),

    #[error("persona identity not found")]
    IdentityNotFound,

    #[error("persona not found")]
    PersonaNotFound,
}
