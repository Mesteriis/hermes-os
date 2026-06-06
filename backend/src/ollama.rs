use std::time::Duration;

use reqwest::Url;
use serde::Deserialize;
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaClientConfig {
    base_url: String,
    chat_model: String,
    embed_model: String,
    timeout_seconds: u64,
}

impl OllamaClientConfig {
    pub fn new(
        base_url: impl Into<String>,
        chat_model: impl Into<String>,
        embed_model: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            chat_model: chat_model.into(),
            embed_model: embed_model.into(),
            timeout_seconds: 120,
        }
    }

    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}

#[derive(Clone)]
pub struct OllamaClient {
    http: reqwest::Client,
    base_url: Url,
    chat_model: String,
    embed_model: String,
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

    pub async fn version(&self) -> Result<String, OllamaError> {
        let response: VersionResponse = self.get_json("/api/version").await?;
        if response.version.trim().is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama version response omitted version".to_owned(),
            ));
        }
        Ok(response.version)
    }

    pub async fn tags(&self) -> Result<Vec<String>, OllamaError> {
        let response: TagsResponse = self.get_json("/api/tags").await?;
        Ok(response
            .models
            .into_iter()
            .map(|model| model.name)
            .filter(|name| !name.trim().is_empty())
            .collect())
    }

    pub async fn validate_required_models(&self) -> Result<(), OllamaError> {
        let tags = self.tags().await?;
        for model in [&self.chat_model, &self.embed_model] {
            if !tags.iter().any(|tag| tag == model) {
                return Err(OllamaError::MissingModel {
                    model: model.to_owned(),
                });
            }
        }
        Ok(())
    }

    pub async fn chat(&self, prompt: &str) -> Result<OllamaChatResult, OllamaError> {
        let body = json!({
            "model": self.chat_model,
            "stream": false,
            "think": false,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
        });
        let response: ChatResponse = self.post_json("/api/chat", &body).await?;
        let content = response
            .message
            .and_then(|message| message.content)
            .ok_or_else(|| {
                OllamaError::Protocol("Ollama chat response omitted assistant content".to_owned())
            })?;
        let content = strip_thinking_content(&content);
        if content.trim().is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama chat response content is empty".to_owned(),
            ));
        }

        Ok(OllamaChatResult {
            model: response.model.unwrap_or_else(|| self.chat_model.clone()),
            content,
            total_duration_ns: response.total_duration,
        })
    }

    pub async fn embed(&self, input: &str) -> Result<OllamaEmbedResult, OllamaError> {
        let body = json!({
            "model": self.embed_model,
            "input": input,
        });
        let response: EmbedResponse = self.post_json("/api/embed", &body).await?;
        let embedding = response
            .embeddings
            .and_then(|mut embeddings| {
                if embeddings.is_empty() {
                    None
                } else {
                    Some(embeddings.remove(0))
                }
            })
            .or(response.embedding)
            .ok_or_else(|| {
                OllamaError::Protocol("Ollama embed response omitted embeddings".to_owned())
            })?;
        if embedding.is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama embed response returned an empty vector".to_owned(),
            ));
        }

        Ok(OllamaEmbedResult {
            model: response.model.unwrap_or_else(|| self.embed_model.clone()),
            embedding,
            total_duration_ns: response.total_duration,
        })
    }

    fn endpoint(&self, path: &str) -> Result<Url, OllamaError> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| OllamaError::InvalidConfig(error.to_string()))
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, OllamaError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.http.get(self.endpoint(path)?).send().await?;
        decode_response(response).await
    }

    async fn post_json<T>(&self, path: &str, body: &Value) -> Result<T, OllamaError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .post(self.endpoint(path)?)
            .json(body)
            .send()
            .await?;
        decode_response(response).await
    }
}

async fn decode_response<T>(response: reqwest::Response) -> Result<T, OllamaError>
where
    T: for<'de> Deserialize<'de>,
{
    let status = response.status();
    if !status.is_success() {
        return Err(OllamaError::Endpoint {
            status: status.as_u16(),
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|error| OllamaError::Protocol(error.to_string()))
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
pub struct OllamaChatResult {
    pub model: String,
    pub content: String,
    pub total_duration_ns: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OllamaEmbedResult {
    pub model: String,
    pub embedding: Vec<f32>,
    pub total_duration_ns: Option<u64>,
}

#[derive(Debug, Error)]
pub enum OllamaError {
    #[error("invalid Ollama client config: {0}")]
    InvalidConfig(String),

    #[error("Ollama endpoint returned HTTP {status}")]
    Endpoint { status: u16 },

    #[error("Ollama model `{model}` is not available")]
    MissingModel { model: String },

    #[error("Ollama protocol error: {0}")]
    Protocol(String),

    #[error("Ollama HTTP request failed")]
    Http(#[from] reqwest::Error),
}

#[derive(Deserialize)]
struct VersionResponse {
    version: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TaggedModel>,
}

#[derive(Deserialize)]
struct TaggedModel {
    name: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    model: Option<String>,
    message: Option<ChatMessage>,
    total_duration: Option<u64>,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct EmbedResponse {
    model: Option<String>,
    embeddings: Option<Vec<Vec<f32>>>,
    embedding: Option<Vec<f32>>,
    total_duration: Option<u64>,
}
