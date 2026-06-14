use thiserror::Error;

use crate::domains::relationships::RelationshipStoreError;

#[derive(Debug, Error)]
pub enum OrgCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error("not found")]
    NotFound,
}
