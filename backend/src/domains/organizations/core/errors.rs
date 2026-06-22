use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum OrgCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
