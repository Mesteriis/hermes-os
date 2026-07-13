use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ObservationValidationError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("unknown observation origin kind: {0}")]
    UnknownOriginKind(String),

    #[error("unknown observation ingestion run status: {0}")]
    UnknownIngestionRunStatus(String),
}
