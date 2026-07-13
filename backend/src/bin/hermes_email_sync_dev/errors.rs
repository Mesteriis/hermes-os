use hermes_communications_postgres::errors::CommunicationIngestionError;
use hermes_hub_backend::integrations::mail::gmail::client::EmailProviderNetworkError;
use hermes_hub_backend::platform::config::ConfigError;
use hermes_hub_backend::platform::secrets::SecretResolutionError;
use hermes_hub_backend::platform::storage::StorageError;
use hermes_hub_backend::workflows::email_sync_pipeline::errors::EmailSyncPipelineError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum DevEmailSyncError {
    #[error("DATABASE_URL is required for email sync dev command")]
    MissingDatabaseUrl,

    #[error("missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("invalid HERMES_EMAIL_SYNC_PROVIDER `{0}`; expected `icloud` or `imap`")]
    InvalidProviderKind(String),

    #[error("Gmail dev sync is not supported by this IMAP-only command")]
    UnsupportedProviderForDevSync,

    #[error("invalid {name} value `{value}`: {message}")]
    InvalidEnv {
        name: &'static str,
        value: String,
        message: &'static str,
    },

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(transparent)]
    ProviderNetwork(#[from] EmailProviderNetworkError),

    #[error(transparent)]
    Pipeline(#[from] EmailSyncPipelineError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
