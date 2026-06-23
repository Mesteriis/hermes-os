use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use super::{TaskCoreError, materialize_task_entity_link_in_transaction};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewPort,
    RelationshipReviewState,
};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

const TASK_RELATIONSHIP_EVIDENCE_EXCERPT: &str =
    "Task relation was recorded through compatibility task relation data.";

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
        .fetch_one(&mut *transaction)
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

        Self::materialize_observation_link_in_transaction(&mut transaction, &relation).await?;
        Self::materialize_relationship_in_transaction(&mut transaction, &relation).await?;
        transaction.commit().await?;

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

    async fn materialize_relationship_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relation: &TaskRelation,
    ) -> Result<(), TaskCoreError> {
        let Ok(target_entity_kind) = RelationshipEntityKind::parse(&relation.entity_type) else {
            return Ok(());
        };

        let observation_id = if let Some(observation_id) = relation
            .source
            .strip_prefix("observation:")
            .filter(|value| !value.is_empty())
        {
            observation_id.to_owned()
        } else {
            let observation = ObservationStore::capture_in_transaction(
                transaction,
                &Self::relation_observation(relation),
            )
            .await?;
            observation.observation_id
        };

        let relationship = NewRelationship {
            source_entity_kind: RelationshipEntityKind::Task,
            source_entity_id: relation.task_id.clone(),
            target_entity_kind,
            target_entity_id: relation.entity_id.clone(),
            relationship_type: relation.relation_type.clone(),
            trust_score: relation.confidence,
            strength_score: relation.confidence,
            confidence: relation.confidence,
            review_state: RelationshipReviewState::UserConfirmed,
            valid_from: None,
            valid_to: None,
            metadata: json!({
                "compatibility_table": "task_relations",
                "compatibility_record_id": relation.id,
                "task_id": relation.task_id,
                "entity_type": relation.entity_type,
                "entity_id": relation.entity_id,
                "source": relation.source,
            }),
        };
        let evidence = NewRelationshipEvidence::observation(observation_id)
            .excerpt(TASK_RELATIONSHIP_EVIDENCE_EXCERPT)
            .metadata(json!({
                "compatibility_table": "task_relations",
                "compatibility_record_id": relation.id,
                "task_id": relation.task_id,
                "entity_type": relation.entity_type,
                "entity_id": relation.entity_id,
            }));

        RelationshipReviewPort::upsert_with_evidence_in_transaction(
            transaction,
            &relationship,
            &[evidence],
        )
        .await?;

        Ok(())
    }

    fn relation_observation(relation: &TaskRelation) -> NewObservation {
        let origin_kind = ObservationOriginKind::parse(&relation.source)
            .unwrap_or(ObservationOriginKind::LocalRuntime);
        NewObservation::new(
            "TASK_MUTATION",
            origin_kind,
            relation.created_at,
            json!({
                "task_id": relation.task_id,
                "entity_type": relation.entity_type,
                "entity_id": relation.entity_id,
                "relation_type": relation.relation_type,
                "source": relation.source,
                "compatibility_record_id": relation.id,
            }),
            format!("task://{}/relation/{}", relation.task_id, relation.id),
        )
        .provenance(json!({
            "captured_by": "tasks.core.relations",
            "operation": "materialize_relationship",
            "source": relation.source,
        }))
    }
}
