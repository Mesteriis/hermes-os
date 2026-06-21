use sqlx::postgres::PgPool;

use crate::app::{ApiError, AppState};
use crate::platform::observations::ObservationStore;

pub(super) fn database_pool(state: &AppState) -> Result<PgPool, ApiError> {
    state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)
        .cloned()
}

pub(super) fn observation_store(state: &AppState) -> Result<ObservationStore, ApiError> {
    Ok(crate::app::api_support::app_store::<ObservationStore>(
        database_pool(state)?,
    ))
}
