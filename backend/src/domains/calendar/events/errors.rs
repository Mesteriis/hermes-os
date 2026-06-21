use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalendarError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] crate::platform::observations::ObservationStoreError),
    #[error("not found")]
    NotFound,
}
