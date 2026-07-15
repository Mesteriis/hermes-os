use thiserror::Error;

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
