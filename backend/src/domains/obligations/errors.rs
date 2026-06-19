use thiserror::Error;

use crate::domains::graph::core::GraphStoreError;
use crate::platform::observations::ObservationStoreError;
use crate::workflows::review_mirror::ReviewMirrorError;

#[derive(Debug, Error)]
pub enum ObligationStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] GraphStoreError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("obligation evidence is required")]
    MissingEvidence,

    #[error("observation obligation evidence must use the same source_id and observation_id")]
    InvalidObservationEvidenceSource,

    #[error("obligation evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("obligation was not found")]
    ObligationNotFound,

    #[error("beneficiary entity kind and id must be provided together")]
    PartialBeneficiary,

    #[error("unknown obligation entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown obligation evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown obligation status stored in database: {0}")]
    UnknownStatus(String),

    #[error("unknown obligation review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("unknown obligation risk state stored in database: {0}")]
    UnknownRiskState(String),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}
