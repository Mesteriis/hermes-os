use thiserror::Error;

use crate::integrations::ollama::client::{OllamaClient, OllamaError};
use crate::integrations::omniroute::client::{OmniRouteClient, OmniRouteError};

#[derive(Clone)]
pub enum AiRuntimeClient {
    Ollama(OllamaClient),
    OmniRoute(OmniRouteClient),
}

impl AiRuntimeClient {
    pub fn runtime_name(&self) -> &'static str {
        match self {
            Self::Ollama(_) => "ollama",
            Self::OmniRoute(_) => "omniroute",
        }
    }

    pub fn chat_model(&self) -> &str {
        match self {
            Self::Ollama(client) => client.chat_model(),
            Self::OmniRoute(client) => client.chat_model(),
        }
    }

    pub fn embedding_model(&self) -> &str {
        match self {
            Self::Ollama(client) => client.embedding_model(),
            Self::OmniRoute(client) => client.embedding_model(),
        }
    }

    pub async fn version(&self) -> Result<Option<String>, AiRuntimeError> {
        match self {
            Self::Ollama(client) => client.version().await.map(Some).map_err(Into::into),
            Self::OmniRoute(_) => Ok(None),
        }
    }

    pub async fn models(&self) -> Result<Vec<String>, AiRuntimeError> {
        match self {
            Self::Ollama(client) => client.tags().await.map_err(Into::into),
            Self::OmniRoute(client) => client.models().await.map_err(Into::into),
        }
    }

    pub async fn validate_required_models(&self) -> Result<(), AiRuntimeError> {
        match self {
            Self::Ollama(client) => client.validate_required_models().await.map_err(Into::into),
            Self::OmniRoute(client) => client.validate_required_models().await.map_err(Into::into),
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<AiChatResult, AiRuntimeError> {
        match self {
            Self::Ollama(client) => {
                let result = client.chat(prompt).await?;
                Ok(AiChatResult {
                    model: result.model,
                    content: result.content,
                    total_duration_ns: result.total_duration_ns,
                })
            }
            Self::OmniRoute(client) => {
                let result = client.chat(prompt).await?;
                Ok(AiChatResult {
                    model: result.model,
                    content: result.content,
                    total_duration_ns: None,
                })
            }
        }
    }

    pub async fn embed(&self, input: &str) -> Result<AiEmbedResult, AiRuntimeError> {
        match self {
            Self::Ollama(client) => {
                let result = client.embed(input).await?;
                Ok(AiEmbedResult {
                    model: result.model,
                    embedding: result.embedding,
                    total_duration_ns: result.total_duration_ns,
                })
            }
            Self::OmniRoute(client) => {
                let result = client.embed(input).await?;
                Ok(AiEmbedResult {
                    model: result.model,
                    embedding: result.embedding,
                    total_duration_ns: None,
                })
            }
        }
    }
}

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

#[derive(Debug, Error)]
pub enum AiRuntimeError {
    #[error(transparent)]
    Ollama(#[from] OllamaError),

    #[error(transparent)]
    OmniRoute(#[from] OmniRouteError),
}
