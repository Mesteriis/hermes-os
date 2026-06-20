use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct CommunicationIngestionStore {
    pub(super) pool: PgPool,
}

impl CommunicationIngestionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> PgPool {
        self.pool.clone()
    }
}
