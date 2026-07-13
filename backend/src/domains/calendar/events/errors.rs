use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalendarError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] hermes_observations_postgres::errors::ObservationStoreError),
    #[error("not found")]
    NotFound,
}
