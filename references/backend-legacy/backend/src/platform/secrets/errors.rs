use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecretReferenceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("unsupported secret kind: {0}")]
    UnsupportedSecretKind(String),

    #[error("unsupported secret store kind: {0}")]
    UnsupportedStoreKind(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SecretResolutionError {
    #[error("secret_ref must not be empty")]
    EmptySecretRef,

    #[error("secret value must not be empty")]
    EmptySecretValue,

    #[error("secret reference was not found: {secret_ref}")]
    MissingSecret { secret_ref: String },

    #[error("secret store kind is not supported by in-memory resolver: {0}")]
    UnsupportedStoreKind(String),

    #[error("secret store operation failed: {message}")]
    StoreFailure { message: String },
}
