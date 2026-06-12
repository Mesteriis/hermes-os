use std::time::Duration;

use reqwest::Url;
use serde::Deserialize;
use serde_json::{Value, json};
use thiserror::Error;

use crate::platform::secrets::ResolvedSecret;

#[derive(Clone)]
pub struct OmniRouteClientConfig {
    base_url: String,
    chat_model: String,
    embed_model: String,
    api_key: ResolvedSecret,
    timeout_seconds: u64,
}

impl OmniRouteClientConfig {
    pub fn new(
        base_url: impl Into<String>,
        chat_model: impl Into<String>,
        embed_model: impl Into<String>,
        api_key: ResolvedSecret,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            chat_model: chat_model.into(),
            embed_model: embed_model.into(),
            api_key,
            timeout_seconds: 120,
        }
    }

    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}

#[derive(Clone)]
pub struct OmniRouteClient {
    http: reqwest::Client,
    base_url: Url,
    chat_model: String,
    embed_model: String,
    api_key: ResolvedSecret,
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

    pub async fn models(&self) -> Result<Vec<String>, OmniRouteError> {
        let response: ModelsResponse = self.get_json("models").await?;
        Ok(response
            .data
            .into_iter()
            .map(|model| model.id)
            .filter(|id| !id.trim().is_empty())
            .collect())
    }

    pub async fn validate_required_models(&self) -> Result<(), OmniRouteError> {
        let models = self.models().await?;
        for model in [&self.chat_model, &self.embed_model] {
            if !models.iter().any(|candidate| candidate == model) {
                return Err(OmniRouteError::MissingModel {
                    model: model.to_owned(),
                });
            }
        }
        Ok(())
    }

    pub async fn chat(&self, prompt: &str) -> Result<OmniRouteChatResult, OmniRouteError> {
        self.chat_with_model(prompt, &self.chat_model).await
    }

    pub async fn chat_with_model(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<OmniRouteChatResult, OmniRouteError> {
        if model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "chat model is empty".to_owned(),
            ));
        }
        let body = json!({
            "model": model,
            "stream": false,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
        });
        let response: ChatCompletionsResponse = self.post_json("chat/completions", &body).await?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| {
                OmniRouteError::Protocol(
                    "OmniRoute chat response omitted assistant content".to_owned(),
                )
            })?;
        let content = strip_thinking_content(&content);
        if content.trim().is_empty() {
            return Err(OmniRouteError::Protocol(
                "OmniRoute chat response content is empty".to_owned(),
            ));
        }

        Ok(OmniRouteChatResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            content,
        })
    }

    pub async fn embed(&self, input: &str) -> Result<OmniRouteEmbedResult, OmniRouteError> {
        self.embed_with_model(input, &self.embed_model).await
    }

    pub async fn embed_with_model(
        &self,
        input: &str,
        model: &str,
    ) -> Result<OmniRouteEmbedResult, OmniRouteError> {
        if model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        let body = json!({
            "model": model,
            "input": input,
        });
        let response: EmbeddingsResponse = self.post_json("embeddings", &body).await?;
        let embedding = response
            .data
            .into_iter()
            .next()
            .map(|item| item.embedding)
            .ok_or_else(|| {
                OmniRouteError::Protocol("OmniRoute embeddings response omitted data".to_owned())
            })?;
        if embedding.is_empty() {
            return Err(OmniRouteError::Protocol(
                "OmniRoute embeddings response returned an empty vector".to_owned(),
            ));
        }

        Ok(OmniRouteEmbedResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            embedding,
        })
    }

    fn endpoint(&self, path: &str) -> Result<Url, OmniRouteError> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| OmniRouteError::InvalidConfig(error.to_string()))
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, OmniRouteError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .get(self.endpoint(path)?)
            .bearer_auth(self.api_key.expose_for_runtime())
            .send()
            .await?;
        decode_response(response).await
    }

    async fn post_json<T>(&self, path: &str, body: &Value) -> Result<T, OmniRouteError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .post(self.endpoint(path)?)
            .bearer_auth(self.api_key.expose_for_runtime())
            .json(body)
            .send()
            .await?;
        decode_response(response).await
    }
}

async fn decode_response<T>(response: reqwest::Response) -> Result<T, OmniRouteError>
where
    T: for<'de> Deserialize<'de>,
{
    let status = response.status();
    if !status.is_success() {
        return Err(OmniRouteError::Endpoint {
            status: status.as_u16(),
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|error| OmniRouteError::Protocol(error.to_string()))
}

fn strip_thinking_content(content: &str) -> String {
    let mut sanitized = content.trim().to_owned();
    while let Some(start) = sanitized.find("<think>") {
        let Some(end_offset) = sanitized[start..].find("</think>") else {
            sanitized.replace_range(start.., "");
            break;
        };
        let end = start + end_offset + "</think>".len();
        sanitized.replace_range(start..end, "");
    }

    if let Some(end) = sanitized.rfind("</think>") {
        sanitized = sanitized[end + "</think>".len()..].to_owned();
    }

    sanitized.trim().to_owned()
}

#[derive(Clone, Debug, PartialEq)]
pub struct OmniRouteChatResult {
    pub model: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OmniRouteEmbedResult {
    pub model: String,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Error)]
pub enum OmniRouteError {
    #[error("invalid OmniRoute client config: {0}")]
    InvalidConfig(String),

    #[error("OmniRoute API key is not configured")]
    MissingApiKey,

    #[error("OmniRoute endpoint returned HTTP {status}")]
    Endpoint { status: u16 },

    #[error("OmniRoute model `{model}` is not available")]
    MissingModel { model: String },

    #[error("OmniRoute protocol error: {0}")]
    Protocol(String),

    #[error("OmniRoute HTTP request failed")]
    Http(#[from] reqwest::Error),
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelItem>,
}

#[derive(Deserialize)]
struct ModelItem {
    id: String,
}

#[derive(Deserialize)]
struct ChatCompletionsResponse {
    model: Option<String>,
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    model: Option<String>,
    data: Vec<EmbeddingItem>,
}

#[derive(Deserialize)]
struct EmbeddingItem {
    embedding: Vec<f32>,
}
