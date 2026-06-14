mod local_state;
mod metadata;
mod queries;
mod upsert;
mod workflow;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct MessageProjectionStore {
    pool: PgPool,
}

impl MessageProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
