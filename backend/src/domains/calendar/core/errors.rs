use thiserror::Error;

use crate::engines::context_packs::ContextPackStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum CalendarCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    ContextPack(#[from] ContextPackStoreError),
    #[error("not found")]
    NotFound,
}
