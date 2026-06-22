use thiserror::Error;

use crate::domains::relationships::RelationshipStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum OrgCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error("not found")]
    NotFound,
}
