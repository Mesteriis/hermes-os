use thiserror::Error;

use crate::platform::events::EventStoreError;

#[derive(Debug, Error)]
pub enum PersonaTrustError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    RiskEngine(#[from] crate::engines::risk::RiskEngineError),

    #[error(transparent)]
    Observation(#[from] crate::platform::observations::ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),
}
