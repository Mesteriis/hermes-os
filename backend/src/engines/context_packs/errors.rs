use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextPackStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("context pack sources are required")]
    MissingSources,

    #[error("unknown context pack kind stored in database: {0}")]
    UnknownContextPackKind(String),

    #[error("unknown context pack source kind stored in database: {0}")]
    UnknownContextPackSourceKind(String),
}
