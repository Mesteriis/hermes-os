use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::graph::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, GraphStoreError,
    NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType, node_id,
};

#[derive(Clone)]
pub struct GraphProjectionService {
    pool: PgPool,
    graph: GraphStore,
}

impl GraphProjectionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            graph: GraphStore::new(pool.clone()),
            pool,
        }
    }

    pub async fn project_from_v1(&self) -> Result<GraphProjectionReport, GraphProjectionError> {
        let mut report = GraphProjectionReport::default();

        for contact in self.list_contacts().await? {
            self.project_contact(&contact, &mut report).await?;
        }
        for message in self.list_messages().await? {
            self.project_message(&message, &mut report).await?;
        }
        for document in self.list_documents().await? {
            self.project_document(&document, &mut report).await?;
        }

        Ok(report)
    }

    async fn list_contacts(&self) -> Result<Vec<ContactRow>, GraphProjectionError> {
        let rows = sqlx::query(
            "SELECT contact_id, display_name, email_address FROM contacts ORDER BY contact_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_contact).collect()
    }

    async fn list_messages(&self) -> Result<Vec<MessageRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
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

    async fn list_documents(&self) -> Result<Vec<DocumentRow>, GraphProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT document_id, document_kind, title, source_fingerprint, imported_at
            FROM documents
            ORDER BY document_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_document).collect()
    }

    async fn project_contact(
        &self,
        contact: &ContactRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let normalized_email = normalize_email_address(&contact.email_address);
        let person = self
            .graph
            .upsert_node(
                &NewGraphNode::new(
                    GraphNodeKind::Person,
                    &contact.contact_id,
                    &contact.display_name,
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
                    person.node_id,
                    email.node_id,
                    RelationshipType::PersonHasEmailAddress,
                    1.0,
                    GraphReviewState::SystemAccepted,
                ),
                &[NewGraphEvidence::new(
                    GraphEvidenceSourceKind::Contact,
                    contact.contact_id.clone(),
                )],
            )
            .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn project_message(
        &self,
        message: &MessageRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        debug_assert!(!message.body_text.trim().is_empty());

        let message_node = self
            .graph
            .upsert_node(
                &NewGraphNode::new(
                    GraphNodeKind::Message,
                    &message.message_id,
                    &message.subject,
                )
                .properties(json!({
                    "account_id": message.account_id,
                    "provider_record_id": message.provider_record_id,
                    "raw_record_id": message.raw_record_id,
                    "occurred_at": message.occurred_at,
                })),
            )
            .await?;
        report.nodes_upserted += 1;

        self.delete_message_edges(&message.message_id).await?;

        let sender = self
            .resolve_message_endpoint(&message.sender, report)
            .await?;
        self.project_message_endpoint(
            sender,
            &message_node.node_id,
            message,
            RelationshipDirection::Sent,
            report,
        )
        .await?;

        for recipient in &message.recipients {
            let recipient = self.resolve_message_endpoint(recipient, report).await?;
            self.project_message_endpoint(
                recipient,
                &message_node.node_id,
                message,
                RelationshipDirection::Received,
                report,
            )
            .await?;
        }

        Ok(())
    }

    async fn delete_message_edges(&self, message_id: &str) -> Result<(), GraphProjectionError> {
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn resolve_message_endpoint(
        &self,
        email_address: &str,
        report: &mut GraphProjectionReport,
    ) -> Result<MessageEndpoint, GraphProjectionError> {
        let normalized_email = normalize_email_address(email_address);
        let contact = self.contact_by_normalized_email(&normalized_email).await?;

        if let Some(contact) = contact {
            return Ok(MessageEndpoint::Person {
                node_id: node_id(GraphNodeKind::Person, &contact.contact_id),
            });
        }

        let email = self
            .graph
            .upsert_node(&NewGraphNode::new(
                GraphNodeKind::EmailAddress,
                &normalized_email,
                &normalized_email,
            ))
            .await?;
        report.nodes_upserted += 1;

        Ok(MessageEndpoint::EmailAddress {
            node_id: email.node_id,
        })
    }

    async fn contact_by_normalized_email(
        &self,
        normalized_email: &str,
    ) -> Result<Option<ContactRow>, GraphProjectionError> {
        let row = sqlx::query(
            "SELECT contact_id, display_name, email_address FROM contacts WHERE email_address = $1",
        )
        .bind(normalized_email)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_contact).transpose()
    }

    async fn project_message_endpoint(
        &self,
        endpoint: MessageEndpoint,
        message_node_id: &str,
        message: &MessageRow,
        direction: RelationshipDirection,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let relationship_type = endpoint.relationship_type(direction);
        self.graph
            .upsert_edge_with_evidence(
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

    async fn project_document(
        &self,
        document: &DocumentRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        self.graph
            .upsert_node(
                &NewGraphNode::new(
                    GraphNodeKind::Document,
                    &document.document_id,
                    &document.title,
                )
                .properties(json!({
                    "document_kind": document.document_kind,
                    "source_fingerprint": document.source_fingerprint,
                    "imported_at": document.imported_at,
                })),
            )
            .await?;
        report.nodes_upserted += 1;

        Ok(())
    }
}

/// Counts deterministic projection operations attempted during a V1 graph projection run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphProjectionReport {
    pub nodes_upserted: usize,
    pub edges_upserted: usize,
    pub evidence_upserted: usize,
}

#[derive(Debug, Error)]
pub enum GraphProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] GraphStoreError),

    #[error("message recipients must be a JSON array of strings")]
    InvalidRecipients,
}

struct ContactRow {
    contact_id: String,
    display_name: String,
    email_address: String,
}

struct MessageRow {
    message_id: String,
    raw_record_id: String,
    account_id: String,
    provider_record_id: String,
    subject: String,
    sender: String,
    recipients: Vec<String>,
    body_text: String,
    occurred_at: Option<DateTime<Utc>>,
}

struct DocumentRow {
    document_id: String,
    document_kind: String,
    title: String,
    source_fingerprint: String,
    imported_at: DateTime<Utc>,
}

enum MessageEndpoint {
    Person { node_id: String },
    EmailAddress { node_id: String },
}

impl MessageEndpoint {
    fn node_id(&self) -> &str {
        match self {
            Self::Person { node_id } | Self::EmailAddress { node_id } => node_id,
        }
    }

    fn relationship_type(&self, direction: RelationshipDirection) -> RelationshipType {
        match (self, direction) {
            (Self::Person { .. }, RelationshipDirection::Sent) => {
                RelationshipType::PersonSentMessage
            }
            (Self::Person { .. }, RelationshipDirection::Received) => {
                RelationshipType::PersonReceivedMessage
            }
            (Self::EmailAddress { .. }, RelationshipDirection::Sent) => {
                RelationshipType::EmailAddressSentMessage
            }
            (Self::EmailAddress { .. }, RelationshipDirection::Received) => {
                RelationshipType::EmailAddressReceivedMessage
            }
        }
    }
}

#[derive(Clone, Copy)]
enum RelationshipDirection {
    Sent,
    Received,
}

fn row_to_contact(row: PgRow) -> Result<ContactRow, GraphProjectionError> {
    Ok(ContactRow {
        contact_id: row.try_get("contact_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
    })
}

fn row_to_message(row: PgRow) -> Result<MessageRow, GraphProjectionError> {
    Ok(MessageRow {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_record_id: row.try_get("provider_record_id")?,
        subject: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        recipients: recipients_from_value(row.try_get("recipients")?)?,
        body_text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

fn row_to_document(row: PgRow) -> Result<DocumentRow, GraphProjectionError> {
    Ok(DocumentRow {
        document_id: row.try_get("document_id")?,
        document_kind: row.try_get("document_kind")?,
        title: row.try_get("title")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        imported_at: row.try_get("imported_at")?,
    })
}

fn recipients_from_value(value: Value) -> Result<Vec<String>, GraphProjectionError> {
    let Some(values) = value.as_array() else {
        return Err(GraphProjectionError::InvalidRecipients);
    };

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(GraphProjectionError::InvalidRecipients)
        })
        .collect()
}

fn message_evidence(message: &MessageRow) -> NewGraphEvidence {
    NewGraphEvidence::new(GraphEvidenceSourceKind::Message, message.message_id.clone())
        .excerpt(message.subject.clone())
        .metadata(json!({
            "raw_record_id": message.raw_record_id,
            "provider_record_id": message.provider_record_id,
        }))
}

fn normalize_email_address(email_address: &str) -> String {
    email_address.trim().to_ascii_lowercase()
}
