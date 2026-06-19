use thiserror::Error;

use crate::domains::persons::enrichment::PersonEnrichmentError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::domains::relationships::RelationshipStoreError;
use crate::platform::observations::ObservationStoreError;
use crate::workflows::review_mirror::ReviewMirrorError;

#[derive(Debug, Error)]
pub enum InvestigatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error(transparent)]
    Memory(#[from] crate::engines::memory::MemoryEngineError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::TimelineEngineError),
    #[error(transparent)]
    Trust(#[from] crate::engines::trust::TrustEngineError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
    #[error("person not found")]
    PersonNotFound,
    #[error("dossier snapshot not found")]
    DossierSnapshotNotFound,
    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidDossierReviewState,
}

impl From<PersonEnrichmentError> for InvestigatorError {
    fn from(error: PersonEnrichmentError) -> Self {
        match error {
            PersonEnrichmentError::NotFound => Self::PersonNotFound,
            PersonEnrichmentError::Sqlx(error) => Self::Sqlx(error),
            PersonEnrichmentError::Relationship(error) => Self::Relationship(error),
            PersonEnrichmentError::Trust(error) => Self::Trust(error),
            PersonEnrichmentError::Observation(error) => Self::Observation(error),
            PersonEnrichmentError::ReviewMirror(error) => Self::ReviewMirror(error),
        }
    }
}

impl From<PersonMemoryError> for InvestigatorError {
    fn from(error: PersonMemoryError) -> Self {
        match error {
            PersonMemoryError::NotFound => Self::PersonNotFound,
            PersonMemoryError::Sqlx(error) => Self::Sqlx(error),
            PersonMemoryError::Memory(error) => Self::Memory(error),
            PersonMemoryError::Timeline(error) => Self::Timeline(error),
            PersonMemoryError::ObservationStore(error) => Self::Observation(error),
        }
    }
}
