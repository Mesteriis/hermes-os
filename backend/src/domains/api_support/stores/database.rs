use super::super::*;

pub(in crate::domains::api_support::stores) fn database_pool(
    state: &AppState,
) -> Result<sqlx::postgres::PgPool, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(pool.clone())
}
