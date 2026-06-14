use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType as GraphRelationshipType,
};

use super::errors::ObligationStoreError;
use super::models::{
    NewObligationEvidence, Obligation, ObligationEntityKind, ObligationReviewState,
};

pub(super) async fn project_obligation_graph_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &Obligation,
    evidence: &[NewObligationEvidence],
) -> Result<(), ObligationStoreError> {
    let obligation_node = NewGraphNode::new(
        GraphNodeKind::Obligation,
        obligation.obligation_id.clone(),
        obligation.statement.clone(),
    )
    .properties(json!({
        "domain": "obligation",
        "obligation_id": obligation.obligation_id,
        "status": obligation.status.as_str(),
        "review_state": obligation.review_state.as_str(),
        "risk_state": obligation.risk_state.as_str(),
    }));
    let stored_obligation_node =
        GraphStore::upsert_node_in_transaction(transaction, &obligation_node).await?;

    project_obligation_entity_edge_in_transaction(
        transaction,
        obligation,
        evidence,
        &stored_obligation_node.node_id,
        obligation.obligated_entity_kind,
        &obligation.obligated_entity_id,
        "obligated_entity",
    )
    .await?;

    if let (Some(beneficiary_entity_kind), Some(beneficiary_entity_id)) = (
        obligation.beneficiary_entity_kind,
        obligation.beneficiary_entity_id.as_deref(),
    ) {
        project_obligation_entity_edge_in_transaction(
            transaction,
            obligation,
            evidence,
            &stored_obligation_node.node_id,
            beneficiary_entity_kind,
            beneficiary_entity_id,
            "beneficiary_entity",
        )
        .await?;
    }

    Ok(())
}

async fn project_obligation_entity_edge_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &Obligation,
    evidence: &[NewObligationEvidence],
    obligation_node_id: &str,
    entity_kind: ObligationEntityKind,
    entity_id: &str,
    link_role: &str,
) -> Result<(), ObligationStoreError> {
    let Some(target_node_kind) = obligation_entity_to_graph_node_kind(entity_kind) else {
        return Ok(());
    };
    let target_node = NewGraphNode::new(target_node_kind, entity_id, entity_id).properties(json!({
        "domain": entity_kind.as_str(),
        "entity_id": entity_id,
    }));
    let stored_target_node =
        GraphStore::upsert_node_in_transaction(transaction, &target_node).await?;

    let graph_edge = NewGraphEdge::new(
        obligation_node_id.to_owned(),
        stored_target_node.node_id,
        GraphRelationshipType::EntityRelationship,
        obligation.confidence,
        obligation_review_state_to_graph_review_state(obligation.review_state),
    )
    .properties(json!({
        "domain": "obligation",
        "obligation_id": obligation.obligation_id,
        "link_role": link_role,
        "status": obligation.status.as_str(),
        "risk_state": obligation.risk_state.as_str(),
    }));
    let graph_evidence = obligation_graph_evidence(obligation, evidence);

    GraphStore::upsert_edge_with_evidence_in_transaction(
        transaction,
        &graph_edge,
        &[graph_evidence],
    )
    .await?;

    Ok(())
}

fn obligation_entity_to_graph_node_kind(
    entity_kind: ObligationEntityKind,
) -> Option<GraphNodeKind> {
    match entity_kind {
        ObligationEntityKind::Persona => Some(GraphNodeKind::Person),
        ObligationEntityKind::Project => Some(GraphNodeKind::Project),
        ObligationEntityKind::Communication => Some(GraphNodeKind::Message),
        ObligationEntityKind::Document => Some(GraphNodeKind::Document),
        ObligationEntityKind::Decision => Some(GraphNodeKind::Decision),
        ObligationEntityKind::Obligation => Some(GraphNodeKind::Obligation),
        ObligationEntityKind::Organization
        | ObligationEntityKind::Task
        | ObligationEntityKind::Event
        | ObligationEntityKind::Knowledge => None,
    }
}

fn obligation_review_state_to_graph_review_state(
    review_state: ObligationReviewState,
) -> GraphReviewState {
    match review_state {
        ObligationReviewState::Suggested => GraphReviewState::Suggested,
        ObligationReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        ObligationReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

fn obligation_graph_evidence(
    obligation: &Obligation,
    evidence: &[NewObligationEvidence],
) -> NewGraphEvidence {
    let first_evidence = evidence.first();
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Obligation,
        obligation.obligation_id.clone(),
    )
    .metadata(json!({
        "domain": "obligation",
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
