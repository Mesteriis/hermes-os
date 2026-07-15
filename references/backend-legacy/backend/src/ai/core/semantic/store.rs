use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct SemanticEmbeddingStore {
    pub(super) pool: PgPool,
}

impl SemanticEmbeddingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
