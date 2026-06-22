use serde_json::json;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphProjectionPort, GraphReviewState, NewGraphEdge,
    NewGraphEvidence, NewGraphNode, RelationshipType,
};

use super::errors::GraphProjectionError;
use super::models::GraphProjectionReport;
use super::service::GraphProjectionService;

pub(super) struct DecisionProjectionRow {
    decision_id: String,
    title: String,
    status: String,
    review_state: String,
    confidence: f64,
}

struct DecisionImpactedEntityRow {
    entity_kind: String,
    entity_id: String,
    impact_type: String,
}

struct DecisionEvidenceRow {
    source_kind: String,
    source_id: String,
    observation_id: Option<String>,
    quote: Option<String>,
}

impl GraphProjectionService {
    pub(super) async fn list_decisions(
        &self,
    ) -> Result<Vec<DecisionProjectionRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                decision_id,
                title,
                status,
                review_state,
                confidence::float8 AS confidence
            FROM decisions
            ORDER BY decision_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(DecisionProjectionRow {
                    decision_id: row.try_get("decision_id")?,
                    title: row.try_get("title")?,
                    status: row.try_get("status")?,
                    review_state: row.try_get("review_state")?,
                    confidence: row.try_get("confidence")?,
                })
            })
            .collect()
    }

    pub(super) async fn project_decision(
        &self,
        decision: &DecisionProjectionRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let impacted_entities = self
            .list_decision_impacted_entities(&decision.decision_id)
            .await?;
        let evidence = self.decision_evidence(&decision.decision_id).await?;

        let mut transaction = self.pool.begin().await?;
        let decision_node = GraphProjectionPort::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Decision,
                &decision.decision_id,
                &decision.title,
            )
            .properties(json!({
                "domain": "decision",
                "decision_id": decision.decision_id,
                "status": decision.status,
                "review_state": decision.review_state,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_decision_edges(&mut transaction, &decision.decision_id)
            .await?;

        let graph_review_state = decision_review_state(&decision.review_state)?;
        let graph_evidence = decision_graph_evidence(decision, evidence.as_ref());

        for entity in impacted_entities {
            let target_node = GraphProjectionPort::upsert_node_in_transaction(
                &mut transaction,
                &NewGraphNode::new(
                    entity_graph_node_kind("decision", &entity.entity_kind)?,
                    &entity.entity_id,
                    &entity.entity_id,
                )
                .properties(json!({
                    "domain": "decision",
                    "entity_kind": entity.entity_kind,
                    "entity_id": entity.entity_id,
                })),
            )
            .await?;
            report.nodes_upserted += 1;

            GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
                &mut transaction,
                &NewGraphEdge::new(
                    decision_node.node_id.clone(),
                    target_node.node_id,
                    RelationshipType::EntityRelationship,
                    decision.confidence,
                    graph_review_state,
                )
                .properties(json!({
                    "domain": "decision",
                    "decision_id": decision.decision_id,
                    "impact_type": entity.impact_type,
                })),
                std::slice::from_ref(&graph_evidence),
            )
            .await?;
            report.edges_upserted += 1;
            report.evidence_upserted += 1;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn list_decision_impacted_entities(
        &self,
        decision_id: &str,
    ) -> Result<Vec<DecisionImpactedEntityRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT entity_kind, entity_id, impact_type
            FROM decision_impacted_entities
            WHERE decision_id = $1
            ORDER BY entity_kind, entity_id
            "#,
        )
        .bind(decision_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(DecisionImpactedEntityRow {
                    entity_kind: row.try_get("entity_kind")?,
                    entity_id: row.try_get("entity_id")?,
                    impact_type: row.try_get("impact_type")?,
                })
            })
            .collect()
    }

    async fn decision_evidence(
        &self,
        decision_id: &str,
    ) -> Result<Option<DecisionEvidenceRow>, GraphProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT source_kind, source_id, observation_id, quote
            FROM decision_evidence
            WHERE decision_id = $1
            ORDER BY created_at, evidence_id
            LIMIT 1
            "#,
        )
        .bind(decision_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(DecisionEvidenceRow {
                source_kind: row.try_get("source_kind")?,
                source_id: row.try_get("source_id")?,
                observation_id: row.try_get("observation_id")?,
                quote: row.try_get("quote")?,
            })
        })
        .transpose()
    }

    async fn delete_decision_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        decision_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE edge_id IN (
                SELECT edge.edge_id
                FROM graph_edges edge
                JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
                WHERE evidence.source_kind = 'decision'
                  AND evidence.source_id = $1
            )
            "#,
        )
        .bind(decision_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

fn decision_graph_evidence(
    decision: &DecisionProjectionRow,
    evidence: Option<&DecisionEvidenceRow>,
) -> NewGraphEvidence {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Decision,
        decision.decision_id.clone(),
    )
    .metadata(json!({ "domain": "decision" }));

    if let Some(evidence) = evidence {
        graph_evidence = graph_evidence.metadata(json!({
            "domain": "decision",
            "source_kind": evidence.source_kind,
            "source_id": evidence.source_id,
        }));
        if let Some(quote) = &evidence.quote {
            graph_evidence = graph_evidence.excerpt(quote.clone());
        }
        if let Some(observation_id) = &evidence.observation_id {
            graph_evidence = graph_evidence.observation_id(observation_id.clone());
        }
    }

    graph_evidence
}

fn decision_review_state(value: &str) -> Result<GraphReviewState, GraphProjectionError> {
    match value {
        "suggested" => Ok(GraphReviewState::Suggested),
        "user_confirmed" => Ok(GraphReviewState::UserConfirmed),
        "user_rejected" => Ok(GraphReviewState::UserRejected),
        _ => Err(GraphProjectionError::InvalidReviewState {
            domain: "decision",
            value: value.to_owned(),
        }),
    }
}

pub(super) fn entity_graph_node_kind(
    domain: &'static str,
    value: &str,
) -> Result<GraphNodeKind, GraphProjectionError> {
    match value {
        "persona" => Ok(GraphNodeKind::Person),
        "organization" => Ok(GraphNodeKind::Organization),
        "project" => Ok(GraphNodeKind::Project),
        "communication" => Ok(GraphNodeKind::Message),
        "document" => Ok(GraphNodeKind::Document),
        "task" => Ok(GraphNodeKind::Task),
        "event" => Ok(GraphNodeKind::Event),
        "decision" => Ok(GraphNodeKind::Decision),
        "obligation" => Ok(GraphNodeKind::Obligation),
        "knowledge" => Ok(GraphNodeKind::Knowledge),
        _ => Err(GraphProjectionError::InvalidEntityKind {
            domain,
            value: value.to_owned(),
        }),
    }
}
