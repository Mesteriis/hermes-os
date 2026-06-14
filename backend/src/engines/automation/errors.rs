use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStoreError};

#[derive(Debug, Error)]
pub enum AutomationError {
    #[error("invalid automation request: {0}")]
    InvalidRequest(String),

    #[error("automation policy was not found")]
    PolicyNotFound,

    #[error("automation policy is disabled")]
    PolicyDisabled,

    #[error("provider chat is not allowed by policy")]
    ChatNotAllowed,

    #[error("automation template variable is missing: {0}")]
    MissingTemplateVariable(String),

    #[error("automation template received undeclared variable: {0}")]
    UndeclaredTemplateVariable(String),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
