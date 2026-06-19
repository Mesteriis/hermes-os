use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersonProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] crate::platform::observations::ObservationStoreError),

    #[error("email address must not be empty")]
    EmptyEmailAddress,

    #[error("invalid email address: {0}")]
    InvalidEmailAddress(String),

    #[error("AI agent id must not be empty")]
    EmptyAiAgentId,

    #[error("invalid AI agent id: {0}")]
    InvalidAiAgentId(String),

    #[error("display name must not be empty")]
    EmptyDisplayName,

    #[error("person was not found: {0}")]
    PersonNotFound(String),

    #[error("invalid persona type: {0}")]
    InvalidPersonaType(String),

    #[error(transparent)]
    Graph(#[from] crate::domains::graph::core::GraphStoreError),
}
