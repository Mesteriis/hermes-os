use thiserror::Error;

use hermes_events_postgres::errors::EventStoreError;
use hermes_observations_postgres::errors::ObservationStoreError;

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
