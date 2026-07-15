use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use super::api::{NewTask, Task, TaskError, TaskStore};
use super::core::errors::TaskCoreError;
use super::core::obligation_links::ObligationTaskLinkStore;

#[derive(Clone)]
pub struct TaskWorkflowCommands {
    pool: PgPool,
}

impl TaskWorkflowCommands {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        task: &NewTask,
    ) -> Result<Task, TaskError> {
        TaskStore::new(self.pool.clone())
            .create_in_transaction(transaction, task)
            .await
    }

    pub async fn link_obligation_fulfillment(
        &self,
        obligation_id: &str,
        task_id: &str,
    ) -> Result<(), TaskCoreError> {
        ObligationTaskLinkStore::new(self.pool.clone())
            .link_fulfillment_task(obligation_id, task_id)
            .await
    }
}
