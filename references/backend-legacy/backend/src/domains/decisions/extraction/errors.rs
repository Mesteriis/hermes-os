use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecisionEngineError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("decided_by entity kind and id must be provided together")]
    PartialDecider,
}
