use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("graph edge confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(f64),

    #[error("graph edges require evidence in the first graph slice")]
    SystemEdgeRequiresEvidence,

    #[error("closed temporal graph edges are unsupported in the first graph slice")]
    TemporalEdgesUnsupported,

    #[error("unknown graph node kind stored in database: {0}")]
    UnknownNodeKind(String),

    #[error("unknown graph relationship type stored in database: {0}")]
    UnknownRelationshipType(String),

    #[error("unknown graph review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("unknown graph evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),
}
