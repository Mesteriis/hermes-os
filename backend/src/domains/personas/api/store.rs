mod ai_agents;
mod email_projection;
mod owner;
mod persona_type;
mod persona_writes;
mod review_projection;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonaProjectionStore {
    pool: PgPool,
}

impl PersonaProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }
}
