use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, GraphStoreError,
    NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType,
};
use crate::domains::relationships::{
    errors::RelationshipStoreError,
    models::{
        NewRelationship, NewRelationshipEvidence, Relationship, RelationshipEntityKind,
        RelationshipReviewState,
    },
    store::RelationshipStore,
};

/// Coordinates two bounded contexts in one PostgreSQL transaction. Relationship
/// persistence remains owned by Relationships; Graph owns the materialized
/// nodes and edge.
#[derive(Clone)]
pub struct RelationshipGraphCoordinator {
    pool: PgPool,
}

impl RelationshipGraphCoordinator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipGraphCoordinatorError> {
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
    ) -> Result<Relationship, RelationshipGraphCoordinatorError> {
        let stored = RelationshipStore::upsert_with_evidence_in_transaction(
            transaction,
            relationship,
            evidence,
        )
        .await?;
        materialize_relationship_graph_in_transaction(transaction, &stored, evidence).await?;
        Ok(stored)
    }

    pub async fn set_review_state_with_observation(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Relationship, RelationshipGraphCoordinatorError> {
        let mut transaction = self.pool.begin().await?;
        let relationship = RelationshipStore::set_review_state_in_transaction(
            &mut transaction,
            relationship_id,
            review_state,
            observation_id,
            metadata.clone(),
        )
        .await?;
        materialize_relationship_graph_review_in_transaction(
            &mut transaction,
            &relationship,
            observation_id,
            metadata.as_ref(),
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
) -> Result<(), RelationshipGraphCoordinatorError> {
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
) -> Result<(), RelationshipGraphCoordinatorError> {
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
) -> Result<(), RelationshipGraphCoordinatorError> {
    let source_node = GraphStore::upsert_node_in_transaction(
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
    let target_node = GraphStore::upsert_node_in_transaction(
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

    GraphStore::upsert_edge_with_evidence_in_transaction(
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

#[derive(Debug, Error)]
pub enum RelationshipGraphCoordinatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    Graph(#[from] GraphStoreError),
}
