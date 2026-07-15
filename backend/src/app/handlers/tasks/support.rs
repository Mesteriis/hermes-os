use sqlx::postgres::PgPool;

use crate::app::error::types::ApiError;
use crate::app::state::AppState;

pub(super) fn database_pool(state: &AppState) -> Result<PgPool, ApiError> {
    state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)
        .cloned()
}
