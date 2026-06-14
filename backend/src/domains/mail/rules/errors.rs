use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailRuleError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid rule: {0}")]
    InvalidRule(&'static str),
}
