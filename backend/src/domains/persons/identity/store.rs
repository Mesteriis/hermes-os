mod candidates;
mod name_merge_candidates;
mod queries;
mod review;
mod review_state;
mod split_candidates;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonIdentityStore {
    pool: PgPool,
}

impl PersonIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }
}
