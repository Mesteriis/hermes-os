mod append;
mod read;
mod replay;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
