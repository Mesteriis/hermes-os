use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventEnvelopeError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("schema_version must be positive")]
    InvalidSchemaVersion,

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error(transparent)]
    Envelope(#[from] EventEnvelopeError),

    #[error("replay position must be non-negative, got {0}")]
    InvalidReplayPosition(i64),
}

impl EventStoreError {
    pub fn is_unique_violation(&self) -> bool {
        match self {
            Self::Sqlx(sqlx::Error::Database(error)) => error.code().as_deref() == Some("23505"),
            _ => false,
        }
    }
}
