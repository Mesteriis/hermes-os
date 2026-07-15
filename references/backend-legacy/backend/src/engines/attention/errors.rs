use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AttentionEngineError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("attention candidate evidence is required for review item {0}")]
    MissingEvidence(String),

    #[error("attention candidate suggested actions are required for review item {0}")]
    MissingSuggestedActions(String),

    #[error("confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(f64),

    #[error("unknown review status for attention candidate: {0}")]
    UnknownReviewStatus(String),
}
