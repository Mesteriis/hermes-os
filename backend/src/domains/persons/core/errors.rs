use thiserror::Error;

use crate::domains::relationships::RelationshipStoreError;

#[derive(Debug, Error)]
pub enum PersonCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error("person identity not found")]
    IdentityNotFound,

    #[error("person persona not found")]
    PersonaNotFound,
}
