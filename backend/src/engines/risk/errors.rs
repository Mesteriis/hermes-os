use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiskEngineError {
    #[error("invalid risk severity `{0}`")]
    InvalidSeverity(String),

    #[error("risk observation {0} must not be empty")]
    EmptyField(&'static str),
}
