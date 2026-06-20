use thiserror::Error;

use crate::domains::communications::messages::MessageProjectionError;
use crate::integrations::ai_runtime::AiRuntimeError;

#[derive(Debug, Error)]
pub enum EmailIntelligenceError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimeError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error("failed to parse AI response: {0}")]
    ParseError(String),
}
