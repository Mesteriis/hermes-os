use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalendarError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
