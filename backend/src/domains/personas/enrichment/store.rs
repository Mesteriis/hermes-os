use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonaEnrichmentStore {
    pub(in crate::domains::personas::enrichment) pool: PgPool,
}

impl PersonaEnrichmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
