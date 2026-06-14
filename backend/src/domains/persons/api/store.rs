mod ai_agents;
mod email_projection;
mod owner;
mod persona_reads;
mod persona_type;
mod persona_writes;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonProjectionStore {
    pool: PgPool,
}

impl PersonProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }
}
