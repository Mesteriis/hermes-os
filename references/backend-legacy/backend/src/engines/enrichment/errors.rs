use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum EnrichmentEngineError {
    #[error("enrichment candidate {0} must not be empty")]
    EmptyField(&'static str),

    #[error("enrichment candidate confidence must be between 0 and 1: {0}")]
    InvalidConfidence(f64),

    #[error("enrichment candidate data must be a JSON object")]
    InvalidData,
}
