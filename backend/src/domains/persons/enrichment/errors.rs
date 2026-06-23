use thiserror::Error;

use crate::engines::trust::TrustEngineError;
use crate::platform::events::EventStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Trust(#[from] TrustEngineError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),

    #[error("person not found")]
    NotFound,
}
