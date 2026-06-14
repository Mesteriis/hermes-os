use thiserror::Error;

use crate::engines::memory::MemoryEngineError;

#[derive(Debug, Error)]
pub enum PersonMemoryError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Memory(#[from] MemoryEngineError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::TimelineEngineError),
    #[error("fact not found")]
    NotFound,
}
