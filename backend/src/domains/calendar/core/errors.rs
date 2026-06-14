use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalendarCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
