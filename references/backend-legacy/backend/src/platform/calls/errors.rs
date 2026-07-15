use thiserror::Error;

#[derive(Debug, Error)]
pub enum CallError {
    #[error("invalid call intelligence request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
