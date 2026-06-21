use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct AiChatResult {
    pub model: String,
    pub content: String,
    pub total_duration_ns: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AiEmbedResult {
    pub model: String,
    pub embedding: Vec<f32>,
    pub total_duration_ns: Option<u64>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{runtime} AI runtime request failed: {message}")]
pub struct AiRuntimePortError {
    pub runtime: String,
    pub message: String,
}

impl AiRuntimePortError {
    pub fn provider(runtime: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            runtime: runtime.into(),
            message: message.into(),
        }
    }
}

pub type SharedAiRuntimePort = Arc<dyn AiRuntimePort>;

pub trait AiRuntimePort: Send + Sync {
    fn runtime_name(&self) -> &'static str;

    fn chat<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiChatResult, AiRuntimePortError>> + Send + 'a>>;

    fn embed_with_model<'a>(
        &'a self,
        input: &'a str,
        model: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiEmbedResult, AiRuntimePortError>> + Send + 'a>>;
}
