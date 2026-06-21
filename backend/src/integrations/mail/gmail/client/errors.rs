use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailProviderNetworkError {
    #[error("invalid provider request field {field}: {message}")]
    InvalidProviderRequest {
        field: &'static str,
        message: &'static str,
    },

    #[error("invalid provider response field {field}: {message}")]
    InvalidProviderResponse {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider response is missing required field: {field}")]
    MissingProviderField { field: &'static str },

    #[error("unexpected provider response: {message}")]
    UnexpectedProviderResponse { message: &'static str },

    #[error("provider operation timed out: {operation}")]
    ProviderTimeout { operation: &'static str },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tls(#[from] async_native_tls::Error),

    #[error(transparent)]
    Imap(#[from] async_imap::error::Error),
}
