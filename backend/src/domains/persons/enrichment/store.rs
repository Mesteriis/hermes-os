use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonEnrichmentStore {
    pub(in crate::domains::persons::enrichment) pool: PgPool,
}

impl PersonEnrichmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
