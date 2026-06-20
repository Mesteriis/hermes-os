use serde_json::json;

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, NewGraphEdge, NewGraphEvidence,
    NewGraphNode, RelationshipType,
};

use super::errors::GraphProjectionError;
use super::helpers::normalize_email_address;
use super::models::{GraphProjectionReport, PersonRow};
use super::rows::row_to_person;
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn list_persons(&self) -> Result<Vec<PersonRow>, GraphProjectionError> {
        let rows = sqlx::query(
            "SELECT person_id, display_name, email_address FROM persons ORDER BY person_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_person).collect()
    }

    pub(super) async fn project_person(
        &self,
        person: &PersonRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let normalized_email = normalize_email_address(&person.email_address);
        let person_node = self
            .graph
            .upsert_node(
                &NewGraphNode::new(
                    GraphNodeKind::Person,
                    &person.person_id,
                    &person.display_name,
                )
                .properties(json!({ "email_address": normalized_email.clone() })),
            )
            .await?;
        report.nodes_upserted += 1;

        let email = self
            .graph
            .upsert_node(&NewGraphNode::new(
                GraphNodeKind::EmailAddress,
                &normalized_email,
                &normalized_email,
            ))
            .await?;
        report.nodes_upserted += 1;

        self.graph
            .upsert_edge_with_evidence(
                &NewGraphEdge::new(
                    person_node.node_id,
                    email.node_id,
                    RelationshipType::PersonHasEmailAddress,
                    1.0,
                    GraphReviewState::SystemAccepted,
                ),
                &[NewGraphEvidence::new(
                    GraphEvidenceSourceKind::Person,
                    person.person_id.clone(),
                )],
            )
            .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }
}
