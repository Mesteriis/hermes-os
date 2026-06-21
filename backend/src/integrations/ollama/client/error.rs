use thiserror::Error;

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
