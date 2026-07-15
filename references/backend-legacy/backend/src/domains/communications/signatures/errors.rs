use thiserror::Error;

#[derive(Debug, Error)]
pub enum CertificateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid cert: {0}")]
    Invalid(&'static str),
}
