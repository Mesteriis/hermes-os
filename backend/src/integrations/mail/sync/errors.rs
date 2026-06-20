use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailSyncPlanError {
    #[error("invalid provider config field {field}: {message}")]
    InvalidProviderConfig {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider account config must not contain secret-like key: {key}")]
    SecretLikeConfigKey { key: String },
}
