use thiserror::Error;

use crate::domains::persons::api::PersonProjectionError;
use crate::integrations::ai_runtime::AiRuntimeError;
use crate::platform::events::EventStoreError;
use crate::platform::observations::ObservationStoreError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("invalid AI request: {0}")]
    InvalidRequest(&'static str),

    #[error("unknown AI agent `{0}`")]
    UnknownAgent(String),

    #[error("invalid semantic source kind `{0}`")]
    InvalidSourceKind(String),

    #[error("embedding dimension must be {expected}, got {actual}")]
    InvalidEmbeddingDimension { expected: usize, actual: usize },

    #[error("AI run was not found")]
    RunNotFound,

    #[error(transparent)]
    Runtime(#[from] AiRuntimeError),

    #[error(transparent)]
    EventEnvelope(#[from] crate::platform::events::EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    PersonProjection(#[from] PersonProjectionError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
