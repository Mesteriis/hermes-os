use thiserror::Error;

use crate::domains::communications::messages::MessageProjectionError;
use crate::platform::ai_runtime::AiRuntimePortError;

#[derive(Debug, Error)]
pub enum EmailIntelligenceError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimePortError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error("failed to parse AI response: {0}")]
    ParseError(String),
}
