use std::collections::HashSet;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use crate::platform::observations::materialize_review_transition_link_in_transaction;
use crate::workflows::review_mirror::sync_relationship_review_state_in_transaction;

use super::errors::RelationshipStoreError;
use super::evidence::link_relationship_entity_in_transaction;
use super::ids::{evidence_id, relationship_id};
use super::models::{
    NewRelationship, NewRelationshipEvidence, Relationship, RelationshipEntityKind,
    RelationshipReviewState,
};
use super::row_mapping::row_to_relationship;
use super::validation::{validate_non_empty, validate_relationship_with_evidence};

#[derive(Clone)]
pub struct RelationshipStore {
    pool: PgPool,
}

impl RelationshipStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipStoreError> {
        validate_relationship_with_evidence(relationship, evidence)?;

        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::upsert_with_evidence_in_transaction(&mut transaction, relationship, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipStoreError> {
        validate_evidence_observations_exist(transaction, evidence).await?;
        let relationship_id = relationship_id(
            relationship.source_entity_kind,
            &relationship.source_entity_id,
            &relationship.relationship_type,
            relationship.target_entity_kind,
            &relationship.target_entity_id,
        );
        let row = sqlx::query(
            r#"
            INSERT INTO relationships (
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score,
                strength_score,
                confidence,
                review_state,
                valid_from,
                valid_to,
                metadata
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                CAST($7 AS NUMERIC(5,4)),
                CAST($8 AS NUMERIC(5,4)),
                CAST($9 AS NUMERIC(5,4)),
                $10,
                $11,
                $12,
                $13
            )
            ON CONFLICT (relationship_id)
            DO UPDATE SET
                trust_score = EXCLUDED.trust_score,
                strength_score = EXCLUDED.strength_score,
                confidence = EXCLUDED.confidence,
                review_state = EXCLUDED.review_state,
                valid_from = EXCLUDED.valid_from,
                valid_to = EXCLUDED.valid_to,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&relationship_id)
        .bind(relationship.source_entity_kind.as_str())
        .bind(&relationship.source_entity_id)
        .bind(relationship.target_entity_kind.as_str())
        .bind(&relationship.target_entity_id)
        .bind(&relationship.relationship_type)
        .bind(relationship.trust_score)
        .bind(relationship.strength_score)
        .bind(relationship.confidence)
        .bind(relationship.review_state.as_str())
        .bind(relationship.valid_from)
        .bind(relationship.valid_to)
        .bind(&relationship.metadata)
        .fetch_one(&mut **transaction)
        .await?;
        let stored = row_to_relationship(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&relationship_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO relationship_evidence (
                    evidence_id,
                    relationship_id,
                    source_kind,
                    source_id,
                    observation_id,
                    excerpt,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (relationship_id, source_kind, source_id)
                DO UPDATE SET
                    observation_id = EXCLUDED.observation_id,
                    excerpt = EXCLUDED.excerpt,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&relationship_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(item.observation_id.as_deref())
            .bind(&item.excerpt)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;

            if let Some(observation_id) = item.observation_id.as_deref() {
                link_relationship_entity_in_transaction(
                    transaction,
                    observation_id,
                    "relationship",
                    relationship_id.clone(),
                    Some("supports"),
                    Some(relationship.confidence),
                    Some(json!({
                        "source_kind": item.source_kind.as_str(),
                        "source_id": item.source_id,
                    })),
                )
                .await?;
            }
        }

        Ok(stored)
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: RelationshipEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Relationship>, RelationshipStoreError> {
        validate_non_empty("entity_id", entity_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            FROM relationships
            WHERE (source_entity_kind = $1 AND source_entity_id = $2)
               OR (target_entity_kind = $1 AND target_entity_id = $2)
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_relationship).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: RelationshipReviewState,
        limit: i64,
    ) -> Result<Vec<Relationship>, RelationshipStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            FROM relationships
            WHERE review_state = $1
            ORDER BY updated_at DESC, relationship_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_relationship).collect()
    }

    pub async fn set_review_state(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
    ) -> Result<Relationship, RelationshipStoreError> {
        self.set_review_state_with_observation(relationship_id, review_state, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Relationship, RelationshipStoreError> {
        validate_non_empty("relationship_id", relationship_id)?;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE relationships
            SET
                review_state = $1,
                updated_at = now()
            WHERE relationship_id = $2
            RETURNING
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(relationship_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(RelationshipStoreError::RelationshipNotFound)?;

        let relationship = row_to_relationship(row)?;
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "relationships",
            "relationship",
            &relationship.relationship_id,
            "review_state",
            relationship.review_state.as_str(),
            metadata,
        )
        .await?;
        sync_relationship_review_state_in_transaction(&mut transaction, &relationship).await?;
        transaction.commit().await?;

        Ok(relationship)
    }
}
async fn validate_evidence_observations_exist(
    transaction: &mut Transaction<'_, Postgres>,
    evidence: &[NewRelationshipEvidence],
) -> Result<(), RelationshipStoreError> {
    let observation_ids: Vec<String> = evidence
        .iter()
        .filter_map(|item| item.observation_id.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if observation_ids.is_empty() {
        return Ok(());
    }

    let stored_observation_ids: HashSet<String> = sqlx::query_scalar::<_, String>(
        r#"
        SELECT observation_id
        FROM observations
        WHERE observation_id = ANY($1)
        "#,
    )
    .bind(&observation_ids)
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .collect();

    for observation_id in observation_ids {
        if !stored_observation_ids.contains(&observation_id) {
            return Err(RelationshipStoreError::ObservationNotFound(observation_id));
        }
    }

    Ok(())
}
