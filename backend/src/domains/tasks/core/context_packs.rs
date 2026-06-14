use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::TaskCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskContextPack {
    pub id: String,
    pub task_id: String,
    pub summary: Option<String>,
    pub source_summary: Option<String>,
    pub open_questions: Value,
    pub blockers: Value,
    pub risks: Value,
    pub suggested_next_action: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskContextPackStore {
    pool: PgPool,
}

impl TaskContextPackStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, task_id: &str) -> Result<Option<TaskContextPack>, TaskCoreError> {
        let row = sqlx::query(
            r#"
            SELECT id::text, task_id, summary, source_summary, open_questions,
                   blockers, risks, suggested_next_action, generated_at, model,
                   created_at, updated_at
            FROM task_context_packs
            WHERE task_id = $1
            ORDER BY generated_at DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(TaskContextPack {
                id: row.try_get("id")?,
                task_id: row.try_get("task_id")?,
                summary: row.try_get("summary")?,
                source_summary: row.try_get("source_summary")?,
                open_questions: row.try_get("open_questions")?,
                blockers: row.try_get("blockers")?,
                risks: row.try_get("risks")?,
                suggested_next_action: row.try_get("suggested_next_action")?,
                generated_at: row.try_get("generated_at")?,
                model: row.try_get("model")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    pub async fn upsert(
        &self,
        task_id: &str,
        summary: Option<&str>,
        questions: Value,
        blockers: Value,
        risks: Value,
        next_action: Option<&str>,
    ) -> Result<TaskContextPack, TaskCoreError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_context_packs (
                task_id, summary, open_questions, blockers, risks, suggested_next_action
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id::text, task_id, summary, source_summary, open_questions,
                      blockers, risks, suggested_next_action, generated_at, model,
                      created_at, updated_at
            "#,
        )
        .bind(task_id)
        .bind(summary)
        .bind(&questions)
        .bind(&blockers)
        .bind(&risks)
        .bind(next_action)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskContextPack {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            summary: row.try_get("summary")?,
            source_summary: row.try_get("source_summary")?,
            open_questions: row.try_get("open_questions")?,
            blockers: row.try_get("blockers")?,
            risks: row.try_get("risks")?,
            suggested_next_action: row.try_get("suggested_next_action")?,
            generated_at: row.try_get("generated_at")?,
            model: row.try_get("model")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
