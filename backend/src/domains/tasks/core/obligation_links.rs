use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::TaskCoreError;

#[derive(Clone)]
pub struct ObligationTaskLinkStore {
    pool: PgPool,
}

impl ObligationTaskLinkStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn link_fulfillment_task(
        &self,
        obligation_id: &str,
        task_id: &str,
    ) -> Result<(), TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        Self::link_fulfillment_task_in_transaction(&mut transaction, obligation_id, task_id)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn link_fulfillment_task_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        obligation_id: &str,
        task_id: &str,
    ) -> Result<(), TaskCoreError> {
        sqlx::query(
            r#"
            INSERT INTO obligation_task_links (
                obligation_id,
                task_id,
                link_kind
            )
            VALUES ($1, $2, 'fulfillment_task')
            ON CONFLICT (obligation_id, task_id, link_kind) DO NOTHING
            "#,
        )
        .bind(obligation_id)
        .bind(task_id)
        .execute(&mut **transaction)
        .await?;
        Ok(())
    }
}
