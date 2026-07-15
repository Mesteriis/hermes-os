use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::TaskCoreError;
use super::observation_links::materialize_task_entity_link_in_transaction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskSubtask {
    pub id: String,
    pub parent_task_id: String,
    pub child_task_id: String,
    pub sort_order: i32,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskSubtaskStore {
    pool: PgPool,
}

impl TaskSubtaskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, parent_id: &str) -> Result<Vec<TaskSubtask>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, parent_task_id, child_task_id, sort_order, source, created_at
            FROM task_subtasks
            WHERE parent_task_id = $1
            ORDER BY sort_order
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskSubtask {
                    id: row.try_get("id")?,
                    parent_task_id: row.try_get("parent_task_id")?,
                    child_task_id: row.try_get("child_task_id")?,
                    sort_order: row.try_get("sort_order")?,
                    source: row.try_get("source")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        parent_id: &str,
        child_id: &str,
        order: i32,
    ) -> Result<TaskSubtask, TaskCoreError> {
        self.add_with_source(parent_id, child_id, order, "manual")
            .await
    }

    pub async fn add_with_source(
        &self,
        parent_id: &str,
        child_id: &str,
        order: i32,
        source: &str,
    ) -> Result<TaskSubtask, TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        let subtask =
            Self::add_in_transaction(&mut transaction, parent_id, child_id, order, source).await?;

        if let Some(observation_id) = subtask
            .source
            .strip_prefix("observation:")
            .filter(|value| !value.is_empty())
        {
            materialize_task_entity_link_in_transaction(
                &mut transaction,
                Some(observation_id),
                "task_subtask",
                &subtask.id,
                None,
                None,
                Some(json!({
                    "parent_task_id": parent_id,
                    "child_task_id": child_id,
                })),
            )
            .await?;
        }

        transaction.commit().await?;
        Ok(subtask)
    }

    async fn add_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        parent_id: &str,
        child_id: &str,
        order: i32,
        source: &str,
    ) -> Result<TaskSubtask, TaskCoreError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_subtasks (parent_task_id, child_task_id, sort_order, source)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (parent_task_id, child_task_id)
            DO UPDATE SET sort_order = $3, source = $4
            RETURNING id::text, parent_task_id, child_task_id, sort_order, source, created_at
            "#,
        )
        .bind(parent_id)
        .bind(child_id)
        .bind(order)
        .bind(source)
        .fetch_one(&mut **transaction)
        .await?;

        Ok(TaskSubtask {
            id: row.try_get("id")?,
            parent_task_id: row.try_get("parent_task_id")?,
            child_task_id: row.try_get("child_task_id")?,
            sort_order: row.try_get("sort_order")?,
            source: row.try_get("source")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
