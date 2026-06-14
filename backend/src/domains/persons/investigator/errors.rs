use thiserror::Error;

use crate::domains::persons::enrichment::PersonEnrichmentError;
use crate::domains::persons::memory::PersonMemoryError;
use crate::domains::relationships::RelationshipStoreError;

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
        }
    }
}
