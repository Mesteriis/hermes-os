use std::future::Future;
use std::pin::Pin;

use thiserror::Error;

use crate::integrations::ollama::client::{OllamaClient, OllamaError};
use crate::integrations::omniroute::client::{OmniRouteClient, OmniRouteError};
use crate::platform::ai_runtime::{AiChatResult, AiEmbedResult, AiRuntimePort, AiRuntimePortError};

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
        self.chat_with_model(prompt, self.chat_model()).await
    }

    pub async fn chat_with_model(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<AiChatResult, AiRuntimeError> {
        match self {
            Self::Ollama(client) => {
                let result = client.chat_with_model(prompt, model).await?;
                Ok(AiChatResult {
                    model: result.model,
                    content: result.content,
                    total_duration_ns: result.total_duration_ns,
                })
            }
            Self::OmniRoute(client) => {
                let result = client.chat_with_model(prompt, model).await?;
                Ok(AiChatResult {
                    model: result.model,
                    content: result.content,
                    total_duration_ns: None,
                })
            }
        }
    }

    pub async fn embed(&self, input: &str) -> Result<AiEmbedResult, AiRuntimeError> {
        self.embed_with_model(input, self.embedding_model()).await
    }

    pub async fn embed_with_model(
        &self,
        input: &str,
        model: &str,
    ) -> Result<AiEmbedResult, AiRuntimeError> {
        match self {
            Self::Ollama(client) => {
                let result = client.embed_with_model(input, model).await?;
                Ok(AiEmbedResult {
                    model: result.model,
                    embedding: result.embedding,
                    total_duration_ns: result.total_duration_ns,
                })
            }
            Self::OmniRoute(client) => {
                let result = client.embed_with_model(input, model).await?;
                Ok(AiEmbedResult {
                    model: result.model,
                    embedding: result.embedding,
                    total_duration_ns: None,
                })
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum AiRuntimeError {
    #[error(transparent)]
    Ollama(#[from] OllamaError),

    #[error(transparent)]
    OmniRoute(#[from] OmniRouteError),
}

impl From<AiRuntimeError> for AiRuntimePortError {
    fn from(error: AiRuntimeError) -> Self {
        match error {
            AiRuntimeError::Ollama(error) => Self::provider("ollama", error.to_string()),
            AiRuntimeError::OmniRoute(error) => Self::provider("omniroute", error.to_string()),
        }
    }
}

impl AiRuntimePort for AiRuntimeClient {
    fn runtime_name(&self) -> &'static str {
        AiRuntimeClient::runtime_name(self)
    }

    fn version<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, AiRuntimePortError>> + Send + 'a>> {
        Box::pin(async move { self.version().await.map_err(Into::into) })
    }

    fn models<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, AiRuntimePortError>> + Send + 'a>> {
        Box::pin(async move { self.models().await.map_err(Into::into) })
    }

    fn chat<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiChatResult, AiRuntimePortError>> + Send + 'a>> {
        Box::pin(async move { self.chat(prompt).await.map_err(Into::into) })
    }

    fn chat_with_model<'a>(
        &'a self,
        prompt: &'a str,
        model: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiChatResult, AiRuntimePortError>> + Send + 'a>> {
        Box::pin(async move {
            self.chat_with_model(prompt, model)
                .await
                .map_err(Into::into)
        })
    }

    fn embed_with_model<'a>(
        &'a self,
        input: &'a str,
        model: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AiEmbedResult, AiRuntimePortError>> + Send + 'a>> {
        Box::pin(async move {
            self.embed_with_model(input, model)
                .await
                .map_err(Into::into)
        })
    }
}
