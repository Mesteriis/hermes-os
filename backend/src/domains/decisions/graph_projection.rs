use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType as GraphRelationshipType,
};

use super::errors::DecisionStoreError;
use super::models::{
    Decision, DecisionEntityKind, DecisionReviewState, NewDecisionEvidence,
    NewDecisionImpactedEntity,
};

pub(super) async fn project_decision_graph_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    decision: &Decision,
    evidence: &[NewDecisionEvidence],
    impacted_entities: &[NewDecisionImpactedEntity],
) -> Result<(), DecisionStoreError> {
    if impacted_entities.is_empty() {
        return Ok(());
    }

    let decision_node = NewGraphNode::new(
        GraphNodeKind::Decision,
        decision.decision_id.clone(),
        decision.title.clone(),
    )
    .properties(json!({
        "domain": "decision",
        "decision_id": decision.decision_id,
        "status": decision.status.as_str(),
        "review_state": decision.review_state.as_str(),
    }));
    let stored_decision_node =
        GraphStore::upsert_node_in_transaction(transaction, &decision_node).await?;

    for impacted_entity in impacted_entities {
        let Some(target_node_kind) =
            decision_entity_to_graph_node_kind(impacted_entity.entity_kind)
        else {
            continue;
        };
        let target_node = NewGraphNode::new(
            target_node_kind,
            impacted_entity.entity_id.clone(),
            impacted_entity.entity_id.clone(),
        )
        .properties(json!({
            "domain": impacted_entity.entity_kind.as_str(),
            "entity_id": impacted_entity.entity_id,
        }));
        let stored_target_node =
            GraphStore::upsert_node_in_transaction(transaction, &target_node).await?;

        let graph_edge = NewGraphEdge::new(
            stored_decision_node.node_id.clone(),
            stored_target_node.node_id,
            GraphRelationshipType::EntityRelationship,
            decision.confidence,
            decision_review_state_to_graph_review_state(decision.review_state),
        )
        .properties(json!({
            "domain": "decision",
            "decision_id": decision.decision_id,
            "impact_type": impacted_entity.impact_type,
        }));
        let graph_evidence = decision_graph_evidence(decision, evidence);

        GraphStore::upsert_edge_with_evidence_in_transaction(
            transaction,
            &graph_edge,
            &[graph_evidence],
        )
        .await?;
    }

    Ok(())
}

fn decision_entity_to_graph_node_kind(entity_kind: DecisionEntityKind) -> Option<GraphNodeKind> {
    match entity_kind {
        DecisionEntityKind::Persona => Some(GraphNodeKind::Person),
        DecisionEntityKind::Project => Some(GraphNodeKind::Project),
        DecisionEntityKind::Communication => Some(GraphNodeKind::Message),
        DecisionEntityKind::Document => Some(GraphNodeKind::Document),
        DecisionEntityKind::Decision => Some(GraphNodeKind::Decision),
        DecisionEntityKind::Organization
        | DecisionEntityKind::Task
        | DecisionEntityKind::Event
        | DecisionEntityKind::Obligation
        | DecisionEntityKind::Knowledge => None,
    }
}

fn decision_review_state_to_graph_review_state(
    review_state: DecisionReviewState,
) -> GraphReviewState {
    match review_state {
        DecisionReviewState::Suggested => GraphReviewState::Suggested,
        DecisionReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        DecisionReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

fn decision_graph_evidence(
    decision: &Decision,
    evidence: &[NewDecisionEvidence],
) -> NewGraphEvidence {
    let first_evidence = evidence.first();
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Decision,
        decision.decision_id.clone(),
    )
    .metadata(json!({
        "domain": "decision",
        "source_kind": first_evidence
            .map(|item| item.source_kind.as_str())
            .unwrap_or("unknown"),
        "source_id": first_evidence
            .map(|item| item.source_id.as_str())
            .unwrap_or("unknown"),
    }));

    if let Some(quote) = first_evidence.and_then(|item| item.quote.as_ref()) {
        graph_evidence = graph_evidence.excerpt(quote.clone());
    }

    graph_evidence
}
