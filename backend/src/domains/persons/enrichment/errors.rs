use thiserror::Error;

use crate::domains::relationships::RelationshipStoreError;
use crate::engines::trust::TrustEngineError;

#[derive(Debug, Error)]
pub enum PersonEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    Trust(#[from] TrustEngineError),

    #[error("person not found")]
    NotFound,
}
