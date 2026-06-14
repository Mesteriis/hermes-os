use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum MemoryEngineError {
    #[error("memory {0} must not be empty")]
    EmptyField(&'static str),
    #[error("memory confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),
    #[error("memory stale threshold days must be greater than zero")]
    InvalidStaleThreshold,
}
