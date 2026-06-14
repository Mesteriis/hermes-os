use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::TaskCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskEvidence {
    pub id: String,
    pub task_id: String,
    pub source_type: String,
    pub source_id: String,
    pub quote: Option<String>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskEvidenceStore {
    pool: PgPool,
}

impl TaskEvidenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, task_id: &str) -> Result<Vec<TaskEvidence>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, task_id, source_type, source_id, quote,
                   confidence::float8 AS confidence, created_at
            FROM task_evidence
            WHERE task_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskEvidence {
                    id: row.try_get("id")?,
                    task_id: row.try_get("task_id")?,
                    source_type: row.try_get("source_type")?,
                    source_id: row.try_get("source_id")?,
                    quote: row.try_get("quote")?,
                    confidence: row.try_get("confidence")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        task_id: &str,
        source_type: &str,
        source_id: &str,
        quote: Option<&str>,
        confidence: Option<f64>,
    ) -> Result<TaskEvidence, TaskCoreError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_evidence (task_id, source_type, source_id, quote, confidence)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id::text, task_id, source_type, source_id, quote,
                      confidence::float8 AS confidence, created_at
            "#,
        )
        .bind(task_id)
        .bind(source_type)
        .bind(source_id)
        .bind(quote)
        .bind(confidence.unwrap_or(1.0))
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskEvidence {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            source_type: row.try_get("source_type")?,
            source_id: row.try_get("source_id")?,
            quote: row.try_get("quote")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
