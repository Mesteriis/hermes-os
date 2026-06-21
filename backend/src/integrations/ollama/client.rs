use std::time::Duration;

use reqwest::Url;

mod catalog;
mod chat;
mod config;
mod embeddings;
mod error;
mod models;
mod responses;
mod sanitization;
mod transport;

pub use config::OllamaClientConfig;
pub use error::OllamaError;
pub use models::{OllamaChatResult, OllamaEmbedResult};

#[derive(Clone)]
pub struct OllamaClient {
    pub(in crate::integrations::ollama::client) http: reqwest::Client,
    pub(in crate::integrations::ollama::client) base_url: Url,
    pub(in crate::integrations::ollama::client) chat_model: String,
    pub(in crate::integrations::ollama::client) embed_model: String,
}

impl OllamaClient {
    pub fn new(config: OllamaClientConfig) -> Result<Self, OllamaError> {
        if config.base_url.trim().is_empty() {
            return Err(OllamaError::InvalidConfig("base URL is empty".to_owned()));
        }
        if config.chat_model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig("chat model is empty".to_owned()));
        }
        if config.embed_model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        if config.timeout_seconds == 0 {
            return Err(OllamaError::InvalidConfig(
                "timeout must be greater than zero".to_owned(),
            ));
        }

        let base_url = Url::parse(&config.base_url)
            .map_err(|error| OllamaError::InvalidConfig(error.to_string()))?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            http,
            base_url,
            chat_model: config.chat_model,
            embed_model: config.embed_model,
        })
    }

    pub fn chat_model(&self) -> &str {
        &self.chat_model
    }

    pub fn embedding_model(&self) -> &str {
        &self.embed_model
    }
}
