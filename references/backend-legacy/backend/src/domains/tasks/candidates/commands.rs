use sqlx::postgres::PgPool;

use super::errors::TaskCandidateError;
use super::store::TaskCandidateStore;

#[derive(Clone)]
pub struct TaskCandidateCommands {
    pool: PgPool,
}

impl TaskCandidateCommands {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_message_candidates(
        &self,
        message_ids: &[String],
    ) -> Result<usize, TaskCandidateError> {
        TaskCandidateStore::new(self.pool.clone())
            .refresh_message_candidates_for_ids(message_ids)
            .await
    }
}
