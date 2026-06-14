use serde_json::json;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType as GraphRelationshipType, node_id,
};

use super::errors::RelationshipStoreError;
use super::models::{Relationship, RelationshipEntityKind, RelationshipReviewState};

struct RelationshipGraphProjection {
    edge: NewGraphEdge,
    evidence: NewGraphEvidence,
}

pub(super) async fn project_relationship_graph_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
) -> Result<(), RelationshipStoreError> {
    let Some(projection) = relationship_graph_projection(relationship) else {
        return Ok(());
    };

    let source_node = relationship_graph_node_in_transaction(
        transaction,
        relationship.source_entity_kind,
        &relationship.source_entity_id,
    )
    .await?;
    let target_node = relationship_graph_node_in_transaction(
        transaction,
        relationship.target_entity_kind,
        &relationship.target_entity_id,
    )
    .await?;

    GraphStore::upsert_node_in_transaction(transaction, &source_node).await?;
    GraphStore::upsert_node_in_transaction(transaction, &target_node).await?;
    GraphStore::upsert_edge_with_evidence_in_transaction(
        transaction,
        &projection.edge,
        &[projection.evidence],
    )
    .await?;

    Ok(())
}

fn relationship_graph_projection(
    relationship: &Relationship,
) -> Option<RelationshipGraphProjection> {
    if relationship.valid_to.is_some() {
        return None;
    }
    let source_node_kind = relationship_graph_node_kind(relationship.source_entity_kind)?;
    let target_node_kind = relationship_graph_node_kind(relationship.target_entity_kind)?;

    let edge = NewGraphEdge::new(
        node_id(source_node_kind, &relationship.source_entity_id),
        node_id(target_node_kind, &relationship.target_entity_id),
        GraphRelationshipType::EntityRelationship,
        relationship.confidence,
        graph_review_state(relationship.review_state),
    )
    .properties(json!({
        "source": "relationships",
        "relationship_id": relationship.relationship_id,
        "relationship_type": relationship.relationship_type,
        "source_entity_kind": relationship.source_entity_kind.as_str(),
        "source_entity_id": relationship.source_entity_id,
        "target_entity_kind": relationship.target_entity_kind.as_str(),
        "target_entity_id": relationship.target_entity_id,
        "trust_score": relationship.trust_score,
        "strength_score": relationship.strength_score,
    }));

    let evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Relationship,
        &relationship.relationship_id,
    )
    .excerpt(relationship.relationship_type.clone())
    .metadata(json!({
        "projection": "relationship_graph",
        "trust_score": relationship.trust_score,
        "strength_score": relationship.strength_score,
        "review_state": relationship.review_state.as_str(),
    }));

    Some(RelationshipGraphProjection { edge, evidence })
}

fn relationship_graph_node_kind(entity_kind: RelationshipEntityKind) -> Option<GraphNodeKind> {
    match entity_kind {
        RelationshipEntityKind::Persona => Some(GraphNodeKind::Person),
        RelationshipEntityKind::Organization => Some(GraphNodeKind::Organization),
        RelationshipEntityKind::Communication => Some(GraphNodeKind::Message),
        RelationshipEntityKind::Document => Some(GraphNodeKind::Document),
        RelationshipEntityKind::Project => Some(GraphNodeKind::Project),
        RelationshipEntityKind::Task => Some(GraphNodeKind::Task),
        RelationshipEntityKind::Event => Some(GraphNodeKind::Event),
        RelationshipEntityKind::Decision => Some(GraphNodeKind::Decision),
        RelationshipEntityKind::Obligation => Some(GraphNodeKind::Obligation),
        RelationshipEntityKind::Knowledge => Some(GraphNodeKind::Knowledge),
    }
}

async fn relationship_graph_node_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    entity_kind: RelationshipEntityKind,
    entity_id: &str,
) -> Result<NewGraphNode, RelationshipStoreError> {
    if entity_kind == RelationshipEntityKind::Persona {
        return persona_graph_node_in_transaction(transaction, entity_id).await;
    }

    let graph_node_kind =
        relationship_graph_node_kind(entity_kind).expect("projection must use supported kind");
    Ok(
        NewGraphNode::new(graph_node_kind, entity_id, entity_id).properties(json!({
            "entity_kind": entity_kind.as_str(),
            "entity_id": entity_id,
        })),
    )
}

async fn persona_graph_node_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: &str,
) -> Result<NewGraphNode, RelationshipStoreError> {
    let row = sqlx::query("SELECT display_name, email_address FROM persons WHERE person_id = $1")
        .bind(persona_id)
        .fetch_optional(&mut **transaction)
        .await?;

    if let Some(row) = row {
        let display_name: String = row.try_get("display_name")?;
        let email_address: String = row.try_get("email_address")?;
        return Ok(
            NewGraphNode::new(GraphNodeKind::Person, persona_id, display_name).properties(json!({
                "persona_id": persona_id,
                "email_address": email_address,
            })),
        );
    }

    Ok(
        NewGraphNode::new(GraphNodeKind::Person, persona_id, persona_id)
            .properties(json!({ "persona_id": persona_id })),
    )
}

fn graph_review_state(review_state: RelationshipReviewState) -> GraphReviewState {
    match review_state {
        RelationshipReviewState::Suggested => GraphReviewState::Suggested,
        RelationshipReviewState::SystemAccepted => GraphReviewState::SystemAccepted,
        RelationshipReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        RelationshipReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use crate::domains::graph::core::{
        GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState,
        RelationshipType as GraphRelationshipType, node_id,
    };

    use super::relationship_graph_projection;
    use crate::domains::relationships::{
        Relationship, RelationshipEntityKind, RelationshipReviewState,
    };

    #[test]
    fn persona_relationship_graph_projection_preserves_domain_relationship_semantics() {
        let now = Utc::now();
        let relationship = Relationship {
            relationship_id: "relationship:v1:test".to_owned(),
            source_entity_kind: RelationshipEntityKind::Persona,
            source_entity_id: "person:v1:email:source@example.com".to_owned(),
            target_entity_kind: RelationshipEntityKind::Persona,
            target_entity_id: "person:v1:email:target@example.com".to_owned(),
            relationship_type: "knows".to_owned(),
            trust_score: 0.77,
            strength_score: 0.58,
            confidence: 0.83,
            review_state: RelationshipReviewState::Suggested,
            valid_from: None,
            valid_to: None,
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };

        let projection = relationship_graph_projection(&relationship)
            .expect("active Persona relationship must have graph projection");

        assert_eq!(
            projection.edge.source_node_id,
            node_id(GraphNodeKind::Person, &relationship.source_entity_id)
        );
        assert_eq!(
            projection.edge.target_node_id,
            node_id(GraphNodeKind::Person, &relationship.target_entity_id)
        );
        assert_eq!(
            projection.edge.relationship_type,
            GraphRelationshipType::EntityRelationship
        );
        assert_eq!(projection.edge.confidence, relationship.confidence);
        assert_eq!(projection.edge.review_state, GraphReviewState::Suggested);
        assert_eq!(
            projection.edge.properties["relationship_id"],
            json!(relationship.relationship_id)
        );
        assert_eq!(
            projection.edge.properties["relationship_type"],
            json!(relationship.relationship_type)
        );
        assert_eq!(
            projection.edge.properties["trust_score"],
            json!(relationship.trust_score)
        );
        assert_eq!(
            projection.edge.properties["strength_score"],
            json!(relationship.strength_score)
        );
        assert_eq!(
            projection.evidence.source_kind,
            GraphEvidenceSourceKind::Relationship
        );
        assert_eq!(projection.evidence.source_id, relationship.relationship_id);
    }
}
