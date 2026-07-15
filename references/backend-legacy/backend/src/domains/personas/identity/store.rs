mod candidates;
mod name_merge_candidates;
mod queries;
mod review;
mod review_state;
mod split_candidates;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonaIdentityReviewStore {
    pool: PgPool,
}

impl PersonaIdentityReviewStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }
}
