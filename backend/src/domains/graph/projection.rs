// This file exceeds 700 lines because it groups the graph projection logic
// with node/edge materialization from events, projection cursor management,
// and graph rebuild orchestration. These are tightly coupled through the
// event-to-graph mapping and cursor checkpoint semantics.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, GraphStoreError,
    NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType, node_id,
};
use crate::domains::projects::core::{
    ProjectMatchedDocument, ProjectMatchedMessage, ProjectProjectionSource, ProjectStore,
    ProjectStoreError,
};
use crate::domains::projects::link_reviews::ProjectLinkReviewState;

const PROJECT_KEYWORD_CONFIDENCE: f64 = 0.75;

#[derive(Clone)]
pub struct GraphProjectionService {
    pool: PgPool,
    graph: GraphStore,
    projects: ProjectStore,
}

impl GraphProjectionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            graph: GraphStore::new(pool.clone()),
            projects: ProjectStore::new(pool.clone()),
            pool,
        }
    }

    pub async fn project_from_v1(&self) -> Result<GraphProjectionReport, GraphProjectionError> {
        let mut report = GraphProjectionReport::default();

        for person in self.list_persons().await? {
            self.project_person(&person, &mut report).await?;
        }
        for message in self.list_messages().await? {
            self.project_message(&message, &mut report).await?;
        }
        for document in self.list_documents().await? {
            self.project_document(&document, &mut report).await?;
        }
        for project in self.projects.graph_projection_projects().await? {
            self.project_project(&project, &mut report).await?;
        }

        Ok(report)
    }

    async fn list_persons(&self) -> Result<Vec<PersonRow>, GraphProjectionError> {
        let rows = sqlx::query(
            "SELECT person_id, display_name, email_address FROM persons ORDER BY person_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_person).collect()
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

    async fn project_person(
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

    async fn project_message(
        &self,
        message: &MessageRow,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let mut transaction = self.pool.begin().await?;
        let message_node = GraphStore::upsert_node_in_transaction(
            &mut transaction,
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

    async fn resolve_message_endpoint(
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
            return Ok(MessageEndpoint::Person {
                node_id: node_id(GraphNodeKind::Person, &person.person_id),
            });
        }

        let email = GraphStore::upsert_node_in_transaction(
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
    ) -> Result<Option<PersonRow>, GraphProjectionError> {
        let row = sqlx::query(
            "SELECT person_id, display_name, email_address FROM persons WHERE email_address = $1",
        )
        .bind(normalized_email)
        .fetch_optional(&mut **transaction)
        .await?;

        row.map(row_to_person).transpose()
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
        GraphStore::upsert_edge_with_evidence_in_transaction(
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

    async fn project_project(
        &self,
        project: &ProjectProjectionSource,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let messages = self
            .projects
            .matching_project_messages(&project.project.project_id)
            .await?;
        let documents = self
            .projects
            .matching_project_documents(&project.project.project_id)
            .await?;

        let mut transaction = self.pool.begin().await?;
        let project_node = GraphStore::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Project,
                &project.project.project_id,
                &project.project.name,
            )
            .properties(json!({
                "kind": project.project.kind,
                "status": project.project.status,
                "description": project.project.description,
                "owner_display_name": project.project.owner_display_name,
                "progress_percent": project.project.progress_percent,
                "start_date": project.project.start_date,
                "target_date": project.project.target_date,
                "keywords": project.keywords,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_project_edges(&mut transaction, &project_node.node_id)
            .await?;

        for message in &messages {
            self.project_project_message(&mut transaction, &project_node.node_id, message, report)
                .await?;
            self.project_project_people(&mut transaction, &project_node.node_id, message, report)
                .await?;
        }

        for document in &documents {
            self.project_project_document(
                &mut transaction,
                &project_node.node_id,
                document,
                report,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn delete_project_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE source_node_id = $1
              AND relationship_type IN (
                  'project_has_message',
                  'project_has_document',
                  'project_involves_person',
                  'project_involves_email_address'
              )
            "#,
        )
        .bind(project_node_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    async fn project_project_message(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        message: &ProjectMatchedMessage,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        GraphStore::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                project_node_id.to_owned(),
                node_id(GraphNodeKind::Message, &message.message_id),
                RelationshipType::ProjectHasMessage,
                project_review_confidence(message.review_state),
                project_review_graph_state(message.review_state),
            )
            .properties(json!({ "match_rule": "project_keyword" })),
            &[project_message_evidence(message)],
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn project_project_document(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        document: &ProjectMatchedDocument,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        GraphStore::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                project_node_id.to_owned(),
                node_id(GraphNodeKind::Document, &document.document_id),
                RelationshipType::ProjectHasDocument,
                project_review_confidence(document.review_state),
                project_review_graph_state(document.review_state),
            )
            .properties(json!({ "match_rule": "project_keyword" })),
            &[project_document_evidence(document)],
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn project_project_people(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        message: &ProjectMatchedMessage,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let mut participant_emails = BTreeSet::new();
        participant_emails.insert(normalize_email_address(&message.sender));
        for recipient in &message.recipients {
            participant_emails.insert(normalize_email_address(recipient));
        }

        for participant_email in participant_emails {
            let endpoint = self
                .resolve_message_endpoint(transaction, &participant_email, report)
                .await?;
            GraphStore::upsert_edge_with_evidence_in_transaction(
                transaction,
                &NewGraphEdge::new(
                    project_node_id.to_owned(),
                    endpoint.node_id().to_owned(),
                    endpoint.project_relationship_type(),
                    project_review_confidence(message.review_state),
                    project_review_graph_state(message.review_state),
                )
                .properties(json!({ "match_rule": "project_keyword" })),
                &[project_message_evidence(message)],
            )
            .await?;
            report.edges_upserted += 1;
            report.evidence_upserted += 1;
        }

        Ok(())
    }
}

/// Counts deterministic projection operations attempted during a V1 graph projection run.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
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

    #[error(transparent)]
    Project(#[from] ProjectStoreError),

    #[error("message recipients must be a JSON array of strings")]
    InvalidRecipients,
}

struct PersonRow {
    person_id: String,
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

    fn project_relationship_type(&self) -> RelationshipType {
        match self {
            Self::Person { .. } => RelationshipType::ProjectInvolvesPerson,
            Self::EmailAddress { .. } => RelationshipType::ProjectInvolvesEmailAddress,
        }
    }
}

#[derive(Clone, Copy)]
enum RelationshipDirection {
    Sent,
    Received,
}

fn row_to_person(row: PgRow) -> Result<PersonRow, GraphProjectionError> {
    Ok(PersonRow {
        person_id: row.try_get("person_id")?,
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

fn project_message_evidence(message: &ProjectMatchedMessage) -> NewGraphEvidence {
    NewGraphEvidence::new(GraphEvidenceSourceKind::Message, message.message_id.clone())
        .excerpt(message.subject.clone())
        .metadata(json!({
            "raw_record_id": message.raw_record_id,
            "account_id": message.account_id,
            "provider_record_id": message.provider_record_id,
            "occurred_at": message.occurred_at,
            "projected_at": message.projected_at,
            "match_rule": "project_keyword",
        }))
}

fn project_document_evidence(document: &ProjectMatchedDocument) -> NewGraphEvidence {
    NewGraphEvidence::new(
        GraphEvidenceSourceKind::Document,
        document.document_id.clone(),
    )
    .excerpt(document.title.clone())
    .metadata(json!({
        "document_kind": document.document_kind,
        "source_fingerprint": document.source_fingerprint,
        "imported_at": document.imported_at,
        "match_rule": "project_keyword",
    }))
}

fn normalize_email_address(email_address: &str) -> String {
    email_address.trim().to_ascii_lowercase()
}

fn project_review_graph_state(review_state: ProjectLinkReviewState) -> GraphReviewState {
    match review_state {
        ProjectLinkReviewState::Suggested => GraphReviewState::Suggested,
        ProjectLinkReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        ProjectLinkReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

fn project_review_confidence(review_state: ProjectLinkReviewState) -> f64 {
    match review_state {
        ProjectLinkReviewState::Suggested => PROJECT_KEYWORD_CONFIDENCE,
        ProjectLinkReviewState::UserConfirmed => 1.0,
        ProjectLinkReviewState::UserRejected => 0.0,
    }
}
