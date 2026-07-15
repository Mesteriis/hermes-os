use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiAuditError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
