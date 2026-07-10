use sqlx::{Postgres, Transaction};

use crate::domains::graph::core::{
    GraphNodeKind, GraphProjectionPort, GraphReviewState, NewGraphEdge, NewGraphNode, node_id,
};

use super::errors::GraphProjectionError;
use super::evidence::message_evidence;
use super::helpers::normalize_email_address;
use super::models::{
    GraphProjectionReport, MessageEndpoint, MessageRow, PersonaRow, RelationshipDirection,
};
use super::rows::{row_to_message, row_to_persona};
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn list_messages(&self) -> Result<Vec<MessageRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                observation_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                recipients,
                body_text,
                occurred_at
            FROM communication_messages
            ORDER BY message_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_message).collect()
    }

    pub(super) async fn project_message(
        &self,
        message: &MessageRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let message_node = GraphProjectionPort::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Message,
                &message.message_id,
                &message.subject,
            )
            .properties(serde_json::json!({
                "account_id": message.account_id,
                "provider_record_id": message.provider_record_id,
                "raw_record_id": message.raw_record_id,
                "observation_id": message.observation_id,
                "occurred_at": message.occurred_at,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_message_edges(&mut transaction, &message.message_id)
            .await?;

        let sender = self
            .resolve_message_endpoint(&mut transaction, &message.sender, report)
            .await?;
        self.project_message_endpoint(
            &mut transaction,
            sender,
            &message_node.node_id,
            message,
            RelationshipDirection::Sent,
            report,
        )
        .await?;

        for recipient in &message.recipients {
            let recipient = self
                .resolve_message_endpoint(&mut transaction, recipient, report)
                .await?;
            self.project_message_endpoint(
                &mut transaction,
                recipient,
                &message_node.node_id,
                message,
                RelationshipDirection::Received,
                report,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn delete_message_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        message_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE edge_id IN (
                SELECT edge.edge_id
                FROM graph_edges edge
                JOIN graph_evidence evidence ON evidence.edge_id = edge.edge_id
                WHERE evidence.source_kind = 'message'
                  AND evidence.source_id = $1
            )
            "#,
        )
        .bind(message_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub(super) async fn resolve_message_endpoint(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        email_address: &str,
        report: &mut GraphProjectionReport,
    ) -> Result<MessageEndpoint, GraphProjectionError> {
        let normalized_email = normalize_email_address(email_address);
        let person = self
            .person_by_normalized_email(transaction, &normalized_email)
            .await?;

        if let Some(person) = person {
            return Ok(MessageEndpoint::Persona {
                node_id: node_id(GraphNodeKind::Persona, &person.persona_id),
            });
        }

        let email = GraphProjectionPort::upsert_node_in_transaction(
            transaction,
            &NewGraphNode::new(
                GraphNodeKind::EmailAddress,
                &normalized_email,
                &normalized_email,
            ),
        )
        .await?;
        report.nodes_upserted += 1;

        Ok(MessageEndpoint::EmailAddress {
            node_id: email.node_id,
        })
    }

    async fn person_by_normalized_email(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        normalized_email: &str,
    ) -> Result<Option<PersonaRow>, GraphProjectionError> {
        let row = sqlx::query(
            "SELECT persona_id, display_name, email_address FROM personas WHERE email_address = $1",
        )
        .bind(normalized_email)
        .fetch_optional(&mut **transaction)
        .await?;

        row.map(row_to_persona).transpose()
    }

    async fn project_message_endpoint(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        endpoint: MessageEndpoint,
        message_node_id: &str,
        message: &MessageRow,
        direction: RelationshipDirection,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let relationship_type = endpoint.relationship_type(direction);
        GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                endpoint.node_id().to_owned(),
                message_node_id.to_owned(),
                relationship_type,
                1.0,
                GraphReviewState::SystemAccepted,
            ),
            &[message_evidence(message)],
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }
}
