use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::TaskCoreError;
use super::observation_links::materialize_task_entity_link_in_transaction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskChecklist {
    pub id: String,
    pub task_id: String,
    pub items: Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskChecklistStore {
    pool: PgPool,
}

impl TaskChecklistStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, task_id: &str) -> Result<Option<TaskChecklist>, TaskCoreError> {
        let row = sqlx::query(
            r#"
            SELECT id::text, task_id, items, source, created_at, updated_at
            FROM task_checklists
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(TaskChecklist {
                id: row.try_get("id")?,
                task_id: row.try_get("task_id")?,
                items: row.try_get("items")?,
                source: row.try_get("source")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    pub async fn set(
        &self,
        task_id: &str,
        items: Value,
        source: &str,
    ) -> Result<TaskChecklist, TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        let checklist = Self::set_in_transaction(&mut transaction, task_id, items, source).await?;

        if let Some(observation_id) = checklist
            .source
            .strip_prefix("observation:")
            .filter(|value| !value.is_empty())
        {
            materialize_task_entity_link_in_transaction(
                &mut transaction,
                Some(observation_id),
                "task_checklist",
                &checklist.id,
                None,
                None,
                Some(json!({
                    "task_id": task_id,
                })),
            )
            .await?;
        }

        transaction.commit().await?;
        Ok(checklist)
    }

    async fn set_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        task_id: &str,
        items: Value,
        source: &str,
    ) -> Result<TaskChecklist, TaskCoreError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_checklists (task_id, items, source)
            VALUES ($1, $2, $3)
            RETURNING id::text, task_id, items, source, created_at, updated_at
            "#,
        )
        .bind(task_id)
        .bind(&items)
        .bind(source)
        .fetch_one(&mut **transaction)
        .await?;

        Ok(TaskChecklist {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            items: row.try_get("items")?,
            source: row.try_get("source")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
