use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use super::{TaskCoreError, materialize_task_entity_link_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskRelation {
    pub id: String,
    pub task_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub relation_type: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskRelationStore {
    pool: PgPool,
}

impl TaskRelationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, task_id: &str) -> Result<Vec<TaskRelation>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, task_id, entity_type, entity_id, relation_type, source,
                   confidence::float8 AS confidence, created_at
            FROM task_relations
            WHERE task_id = $1
            ORDER BY relation_type
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskRelation {
                    id: row.try_get("id")?,
                    task_id: row.try_get("task_id")?,
                    entity_type: row.try_get("entity_type")?,
                    entity_id: row.try_get("entity_id")?,
                    relation_type: row.try_get("relation_type")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn link(
        &self,
        task_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
        source: &str,
    ) -> Result<TaskRelation, TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        let relation = Self::link_in_transaction(
            &mut transaction,
            task_id,
            entity_type,
            entity_id,
            relation_type,
            source,
        )
        .await?;
        transaction.commit().await?;

        Ok(relation)
    }

    pub(crate) async fn link_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        task_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
        source: &str,
    ) -> Result<TaskRelation, TaskCoreError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_relations (task_id, entity_type, entity_id, relation_type, source)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT DO NOTHING
            RETURNING id::text, task_id, entity_type, entity_id, relation_type,
                      source, confidence::float8 AS confidence, created_at
            "#,
        )
        .bind(task_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(relation_type)
        .bind(source)
        .fetch_one(&mut **transaction)
        .await?;
        let relation = TaskRelation {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        };

        Self::materialize_observation_link_in_transaction(transaction, &relation).await?;

        Ok(relation)
    }

    async fn materialize_observation_link_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relation: &TaskRelation,
    ) -> Result<(), TaskCoreError> {
        let Some(observation_id) = relation
            .source
            .strip_prefix("observation:")
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };

        materialize_task_entity_link_in_transaction(
            transaction,
            Some(observation_id),
            "task_relation",
            &relation.id,
            None,
            None,
            Some(json!({
                "task_id": relation.task_id,
                "entity_type": relation.entity_type,
                "entity_id": relation.entity_id,
            })),
        )
        .await?;

        Ok(())
    }
}
