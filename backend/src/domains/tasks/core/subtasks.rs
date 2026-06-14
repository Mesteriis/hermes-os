use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::TaskCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskSubtask {
    pub id: String,
    pub parent_task_id: String,
    pub child_task_id: String,
    pub sort_order: i32,
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
            SELECT id::text, parent_task_id, child_task_id, sort_order, created_at
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
        let row = sqlx::query(
            r#"
            INSERT INTO task_subtasks (parent_task_id, child_task_id, sort_order)
            VALUES ($1, $2, $3)
            ON CONFLICT (parent_task_id, child_task_id)
            DO UPDATE SET sort_order = $3
            RETURNING id::text, parent_task_id, child_task_id, sort_order, created_at
            "#,
        )
        .bind(parent_id)
        .bind(child_id)
        .bind(order)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskSubtask {
            id: row.try_get("id")?,
            parent_task_id: row.try_get("parent_task_id")?,
            child_task_id: row.try_get("child_task_id")?,
            sort_order: row.try_get("sort_order")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
