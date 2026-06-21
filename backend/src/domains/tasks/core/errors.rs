use thiserror::Error;

use crate::engines::context_packs::ContextPackStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum TaskCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ContextPack(#[from] ContextPackStoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
