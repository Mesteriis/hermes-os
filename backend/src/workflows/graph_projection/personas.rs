use serde_json::json;

use crate::domains::graph::core::models::{
    GraphEvidenceSourceKind, GraphReviewState, NewGraphEdge, NewGraphEvidence, NewGraphNode,
    RelationshipType,
};
use crate::platform::graph::GraphNodeKind;

use super::errors::GraphProjectionError;
use super::helpers::normalize_email_address;
use super::models::{GraphProjectionReport, PersonaRow};
use super::rows::row_to_persona;
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn list_personas(&self) -> Result<Vec<PersonaRow>, GraphProjectionError> {
        let rows = sqlx::query(
            "SELECT persona_id, display_name, email_address FROM personas ORDER BY persona_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_persona).collect()
    }

    pub(super) async fn project_persona(
        &self,
        persona: &PersonaRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let normalized_email = persona
            .email_address
            .as_deref()
            .map(normalize_email_address)
            .filter(|email| !email.is_empty());
        let persona_node = self
            .graph
            .upsert_node(
                &NewGraphNode::new(
                    GraphNodeKind::Persona,
                    &persona.persona_id,
                    &persona.display_name,
                )
                .properties(json!({ "email_address": normalized_email.clone() })),
            )
            .await?;
        report.nodes_upserted += 1;

        let Some(normalized_email) = normalized_email else {
            return Ok(());
        };

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
                    persona_node.node_id,
                    email.node_id,
                    RelationshipType::PersonaHasEmailAddress,
                    1.0,
                    GraphReviewState::SystemAccepted,
                ),
                &[NewGraphEvidence::new(
                    GraphEvidenceSourceKind::Persona,
                    persona.persona_id.clone(),
                )],
            )
            .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }
}
