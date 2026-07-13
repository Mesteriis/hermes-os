use sqlx::postgres::PgPool;

use crate::app::{ApiError, AppState};
use hermes_observations_postgres::store::ObservationStore;

pub(super) fn database_pool(state: &AppState) -> Result<PgPool, ApiError> {
    state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)
        .cloned()
}

pub(super) fn observation_store(state: &AppState) -> Result<ObservationStore, ApiError> {
    Ok(crate::app::api_support::stores::domain_stores::app_store::<
        ObservationStore,
    >(database_pool(state)?))
}
