use serde_json::json;
use sqlx::{Postgres, Row, Transaction};

use crate::domains::graph::core::models::{
    GraphEvidenceSourceKind, GraphReviewState, NewGraphEdge, NewGraphEvidence, NewGraphNode,
    RelationshipType,
};
use crate::domains::graph::ports::GraphProjectionPort;
use crate::platform::graph::GraphNodeKind;

use super::decisions::entity_graph_node_kind;
use super::errors::GraphProjectionError;
use super::models::GraphProjectionReport;
use super::service::GraphProjectionService;

pub(super) struct ObligationProjectionRow {
    obligation_id: String,
    obligated_entity_kind: String,
    obligated_entity_id: String,
    beneficiary_entity_kind: Option<String>,
    beneficiary_entity_id: Option<String>,
    statement: String,
    status: String,
    review_state: String,
    confidence: f64,
}

struct ObligationEvidenceRow {
    source_kind: String,
    source_id: String,
    observation_id: Option<String>,
    quote: Option<String>,
}

impl GraphProjectionService {
    pub(super) async fn list_obligations(
        &self,
    ) -> Result<Vec<ObligationProjectionRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                confidence::float8 AS confidence
            FROM obligations
            ORDER BY obligation_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ObligationProjectionRow {
                    obligation_id: row.try_get("obligation_id")?,
                    obligated_entity_kind: row.try_get("obligated_entity_kind")?,
                    obligated_entity_id: row.try_get("obligated_entity_id")?,
                    beneficiary_entity_kind: row.try_get("beneficiary_entity_kind")?,
                    beneficiary_entity_id: row.try_get("beneficiary_entity_id")?,
                    statement: row.try_get("statement")?,
                    status: row.try_get("status")?,
                    review_state: row.try_get("review_state")?,
                    confidence: row.try_get("confidence")?,
                })
            })
            .collect()
    }

    pub(super) async fn project_obligation(
        &self,
        obligation: &ObligationProjectionRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let evidence = self.obligation_evidence(&obligation.obligation_id).await?;

        let mut transaction = self.pool.begin().await?;
        let obligation_node = GraphProjectionPort::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Obligation,
                &obligation.obligation_id,
                &obligation.statement,
            )
            .properties(json!({
                "domain": "obligation",
                "obligation_id": obligation.obligation_id,
                "status": obligation.status,
                "review_state": obligation.review_state,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_obligation_edges(&mut transaction, &obligation.obligation_id)
            .await?;

        let graph_review_state = obligation_review_state(&obligation.review_state)?;
        let graph_evidence = obligation_graph_evidence(obligation, evidence.as_ref());

        self.project_obligation_entity_edge(
            &mut transaction,
            &obligation_node.node_id,
            &obligation.obligated_entity_kind,
            &obligation.obligated_entity_id,
            "obligated_entity",
            obligation.confidence,
            graph_review_state,
            &graph_evidence,
            report,
        )
        .await?;

        if let (Some(beneficiary_entity_kind), Some(beneficiary_entity_id)) = (
            obligation.beneficiary_entity_kind.as_deref(),
            obligation.beneficiary_entity_id.as_deref(),
        ) {
            self.project_obligation_entity_edge(
                &mut transaction,
                &obligation_node.node_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                "beneficiary_entity",
                obligation.confidence,
                graph_review_state,
                &graph_evidence,
                report,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn project_obligation_entity_edge(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        obligation_node_id: &str,
        entity_kind: &str,
        entity_id: &str,
        link_role: &str,
        confidence: f64,
        review_state: GraphReviewState,
        graph_evidence: &NewGraphEvidence,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let target_node = GraphProjectionPort::upsert_node_in_transaction(
            transaction,
            &NewGraphNode::new(
                entity_graph_node_kind("obligation", entity_kind)?,
                entity_id,
                entity_id,
            )
            .properties(json!({
                "domain": "obligation",
                "entity_kind": entity_kind,
                "entity_id": entity_id,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                obligation_node_id.to_owned(),
                target_node.node_id,
                RelationshipType::EntityRelationship,
                confidence,
                review_state,
            )
            .properties(json!({
                "domain": "obligation",
                "link_role": link_role,
            })),
            std::slice::from_ref(graph_evidence),
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn obligation_evidence(
        &self,
        obligation_id: &str,
    ) -> Result<Option<ObligationEvidenceRow>, GraphProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT source_kind, source_id, observation_id, quote
            FROM obligation_evidence
            WHERE obligation_id = $1
            ORDER BY created_at, evidence_id
            LIMIT 1
            "#,
        )
        .bind(obligation_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(ObligationEvidenceRow {
                source_kind: row.try_get("source_kind")?,
                source_id: row.try_get("source_id")?,
                observation_id: row.try_get("observation_id")?,
                quote: row.try_get("quote")?,
            })
        })
        .transpose()
    }

    async fn delete_obligation_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        obligation_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE edge_id IN (
                SELECT edge.edge_id
                FROM graph_edges edge
                JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
                WHERE evidence.source_kind = 'obligation'
                  AND evidence.source_id = $1
            )
            "#,
        )
        .bind(obligation_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

fn obligation_graph_evidence(
    obligation: &ObligationProjectionRow,
    evidence: Option<&ObligationEvidenceRow>,
) -> NewGraphEvidence {
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Obligation,
        obligation.obligation_id.clone(),
    )
    .metadata(json!({ "domain": "obligation" }));

    if let Some(evidence) = evidence {
        graph_evidence = graph_evidence.metadata(json!({
            "domain": "obligation",
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

fn obligation_review_state(value: &str) -> Result<GraphReviewState, GraphProjectionError> {
    match value {
        "suggested" => Ok(GraphReviewState::Suggested),
        "user_confirmed" => Ok(GraphReviewState::UserConfirmed),
        "user_rejected" => Ok(GraphReviewState::UserRejected),
        _ => Err(GraphProjectionError::InvalidReviewState {
            domain: "obligation",
            value: value.to_owned(),
        }),
    }
}
