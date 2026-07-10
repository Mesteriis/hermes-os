use std::collections::HashSet;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphProjectionPort, GraphReviewState, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType,
};
use crate::platform::observations::materialize_review_transition_link_in_transaction;

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

        materialize_relationship_graph_in_transaction(transaction, &stored, evidence).await?;

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
        materialize_relationship_graph_review_in_transaction(
            &mut transaction,
            &relationship,
            observation_id,
            metadata.as_ref(),
        )
        .await?;
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
        transaction.commit().await?;

        Ok(relationship)
    }
}

async fn materialize_relationship_graph_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
    evidence: &[NewRelationshipEvidence],
) -> Result<(), RelationshipStoreError> {
    let graph_evidence = relationship_graph_evidence(relationship, evidence);
    materialize_relationship_graph_with_evidence_in_transaction(
        transaction,
        relationship,
        std::slice::from_ref(&graph_evidence),
    )
    .await
}

async fn materialize_relationship_graph_review_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
    observation_id: Option<&str>,
    metadata: Option<&Value>,
) -> Result<(), RelationshipStoreError> {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Relationship,
        relationship.relationship_id.clone(),
    )
    .metadata(json!({
        "relationship_id": relationship.relationship_id,
        "relationship_type": relationship.relationship_type,
        "review_state": relationship.review_state.as_str(),
    }));
    if let Some(observation_id) = observation_id {
        graph_evidence = graph_evidence.observation_id(observation_id.to_owned());
    }
    if let Some(metadata) = metadata {
        graph_evidence = graph_evidence.metadata(json!({
            "relationship_id": relationship.relationship_id,
            "relationship_type": relationship.relationship_type,
            "review_state": relationship.review_state.as_str(),
            "review_transition": metadata.clone(),
        }));
    }

    materialize_relationship_graph_with_evidence_in_transaction(
        transaction,
        relationship,
        std::slice::from_ref(&graph_evidence),
    )
    .await
}

async fn materialize_relationship_graph_with_evidence_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
    graph_evidence: &[NewGraphEvidence],
) -> Result<(), RelationshipStoreError> {
    let source_node = GraphProjectionPort::upsert_node_in_transaction(
        transaction,
        &NewGraphNode::new(
            graph_node_kind(relationship.source_entity_kind),
            relationship.source_entity_id.clone(),
            relationship.source_entity_id.clone(),
        )
        .properties(json!({
            "entity_kind": relationship.source_entity_kind.as_str(),
            "entity_id": relationship.source_entity_id,
        })),
    )
    .await?;
    let target_node = GraphProjectionPort::upsert_node_in_transaction(
        transaction,
        &NewGraphNode::new(
            graph_node_kind(relationship.target_entity_kind),
            relationship.target_entity_id.clone(),
            relationship.target_entity_id.clone(),
        )
        .properties(json!({
            "entity_kind": relationship.target_entity_kind.as_str(),
            "entity_id": relationship.target_entity_id,
        })),
    )
    .await?;

    GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
        transaction,
        &NewGraphEdge::new(
            source_node.node_id,
            target_node.node_id,
            RelationshipType::EntityRelationship,
            relationship.confidence,
            graph_review_state(relationship.review_state),
        )
        .properties(json!({
            "relationship_id": relationship.relationship_id,
            "relationship_type": relationship.relationship_type,
            "source_entity_kind": relationship.source_entity_kind.as_str(),
            "source_entity_id": relationship.source_entity_id,
            "target_entity_kind": relationship.target_entity_kind.as_str(),
            "target_entity_id": relationship.target_entity_id,
            "trust_score": relationship.trust_score,
            "strength_score": relationship.strength_score,
        })),
        graph_evidence,
    )
    .await?;

    Ok(())
}

fn graph_node_kind(entity_kind: RelationshipEntityKind) -> GraphNodeKind {
    match entity_kind {
        RelationshipEntityKind::Persona => GraphNodeKind::Persona,
        RelationshipEntityKind::Organization => GraphNodeKind::Organization,
        RelationshipEntityKind::Project => GraphNodeKind::Project,
        RelationshipEntityKind::Communication => GraphNodeKind::Message,
        RelationshipEntityKind::Document => GraphNodeKind::Document,
        RelationshipEntityKind::Task => GraphNodeKind::Task,
        RelationshipEntityKind::Event => GraphNodeKind::Event,
        RelationshipEntityKind::Decision => GraphNodeKind::Decision,
        RelationshipEntityKind::Obligation => GraphNodeKind::Obligation,
        RelationshipEntityKind::Knowledge => GraphNodeKind::Knowledge,
    }
}

fn graph_review_state(review_state: RelationshipReviewState) -> GraphReviewState {
    match review_state {
        RelationshipReviewState::Suggested => GraphReviewState::Suggested,
        RelationshipReviewState::SystemAccepted => GraphReviewState::SystemAccepted,
        RelationshipReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        RelationshipReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

fn relationship_graph_evidence(
    relationship: &Relationship,
    evidence: &[NewRelationshipEvidence],
) -> NewGraphEvidence {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Relationship,
        relationship.relationship_id.clone(),
    )
    .metadata(json!({
        "relationship_id": relationship.relationship_id,
        "relationship_type": relationship.relationship_type,
    }));

    if let Some(item) = evidence.first() {
        if let Some(excerpt) = item.excerpt.clone() {
            graph_evidence = graph_evidence.excerpt(excerpt);
        }
        if let Some(observation_id) = item.observation_id.clone() {
            graph_evidence = graph_evidence.observation_id(observation_id);
        }
    }

    graph_evidence
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
