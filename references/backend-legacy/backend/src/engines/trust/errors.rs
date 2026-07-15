use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TrustEngineError {
    #[error("trust signal {0} must not be empty")]
    EmptyField(&'static str),

    #[error("trust signal confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),
}
