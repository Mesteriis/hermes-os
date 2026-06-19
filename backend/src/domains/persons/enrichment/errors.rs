use thiserror::Error;

use crate::domains::relationships::RelationshipStoreError;
use crate::engines::trust::TrustEngineError;
use crate::platform::observations::ObservationStoreError;
use crate::workflows::review_mirror::ReviewMirrorError;

#[derive(Debug, Error)]
pub enum PersonEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    Trust(#[from] TrustEngineError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error("person not found")]
    NotFound,
}
