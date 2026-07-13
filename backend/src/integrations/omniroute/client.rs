use std::time::Duration;

use reqwest::Url;

mod catalog;
mod chat;
pub mod config;
mod embeddings;
pub mod error;
pub(crate) mod models;
mod transport;

use config::OmniRouteClientConfig;
use error::OmniRouteError;

#[derive(Clone)]
pub struct OmniRouteClient {
    http: reqwest::Client,
    base_url: Url,
    chat_model: String,
    embed_model: String,
    api_key: crate::platform::secrets::ResolvedSecret,
}

impl OmniRouteClient {
    pub fn new(config: OmniRouteClientConfig) -> Result<Self, OmniRouteError> {
        if config.base_url.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "base URL is empty".to_owned(),
            ));
        }
        if config.chat_model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "chat model is empty".to_owned(),
            ));
        }
        if config.embed_model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        if config.timeout_seconds == 0 {
            return Err(OmniRouteError::InvalidConfig(
                "timeout must be greater than zero".to_owned(),
            ));
        }

        let mut base_url = config.base_url.trim_end_matches('/').to_owned();
        base_url.push('/');
        let base_url = Url::parse(&base_url)
            .map_err(|error| OmniRouteError::InvalidConfig(error.to_string()))?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            http,
            base_url,
            chat_model: config.chat_model,
            embed_model: config.embed_model,
            api_key: config.api_key,
        })
    }

    pub fn chat_model(&self) -> &str {
        &self.chat_model
    }

    pub fn embedding_model(&self) -> &str {
        &self.embed_model
    }
}
