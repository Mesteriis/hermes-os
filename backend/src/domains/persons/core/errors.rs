use thiserror::Error;

use crate::domains::relationships::RelationshipStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("person identity not found")]
    IdentityNotFound,

    #[error("person persona not found")]
    PersonaNotFound,
}
