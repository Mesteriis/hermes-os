use thiserror::Error;

use crate::engines::memory::MemoryEngineError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonMemoryError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Memory(#[from] MemoryEngineError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::TimelineEngineError),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
    #[error("fact not found")]
    NotFound,
}
