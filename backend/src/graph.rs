use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

pub const GRAPH_NEIGHBORHOOD_EDGE_LIMIT: i64 = 100;

#[derive(Clone)]
pub struct GraphStore {
    pool: PgPool,
}

impl GraphStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_node(&self, node: &NewGraphNode) -> Result<GraphNode, GraphStoreError> {
        node.validate()?;
        let node_id = node_id(node.node_kind, &node.stable_key);
        let row = sqlx::query(
            r#"
            INSERT INTO graph_nodes (node_id, node_kind, stable_key, label, properties)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = EXCLUDED.properties,
                updated_at = now()
            RETURNING node_id, node_kind, stable_key, label, properties, created_at, updated_at
            "#,
        )
        .bind(&node_id)
        .bind(node.node_kind.as_str())
        .bind(&node.stable_key)
        .bind(&node.label)
        .bind(&node.properties)
        .fetch_one(&self.pool)
        .await?;

        row_to_node(row)
    }

    pub async fn upsert_edge_with_evidence(
        &self,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphStoreError> {
        validate_edge_with_evidence(edge, evidence)?;
        let mut transaction = self.pool.begin().await?;
        let stored_edge =
            Self::upsert_edge_with_evidence_in_transaction(&mut transaction, edge, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored_edge)
    }

    pub(crate) async fn upsert_node_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        node: &NewGraphNode,
    ) -> Result<GraphNode, GraphStoreError> {
        node.validate()?;
        let node_id = node_id(node.node_kind, &node.stable_key);
        let row = sqlx::query(
            r#"
            INSERT INTO graph_nodes (node_id, node_kind, stable_key, label, properties)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = EXCLUDED.properties,
                updated_at = now()
            RETURNING node_id, node_kind, stable_key, label, properties, created_at, updated_at
            "#,
        )
        .bind(&node_id)
        .bind(node.node_kind.as_str())
        .bind(&node.stable_key)
        .bind(&node.label)
        .bind(&node.properties)
        .fetch_one(&mut **transaction)
        .await?;

        row_to_node(row)
    }

    pub(crate) async fn upsert_edge_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        edge: &NewGraphEdge,
        evidence: &[NewGraphEvidence],
    ) -> Result<GraphEdge, GraphStoreError> {
        validate_edge_with_evidence(edge, evidence)?;

        let edge_id = edge_id(
            &edge.source_node_id,
            edge.relationship_type,
            &edge.target_node_id,
        );
        let row = sqlx::query(
            r#"
            INSERT INTO graph_edges (
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence,
                review_state,
                properties,
                valid_from,
                valid_to
            )
            VALUES ($1, $2, $3, $4, CAST($5 AS NUMERIC(5,4)), $6, $7, $8, $9)
            ON CONFLICT (source_node_id, target_node_id, relationship_type) WHERE valid_to IS NULL
            DO UPDATE SET
                confidence = EXCLUDED.confidence,
                review_state = EXCLUDED.review_state,
                properties = EXCLUDED.properties,
                valid_from = EXCLUDED.valid_from,
                valid_to = EXCLUDED.valid_to,
                updated_at = now()
            RETURNING
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence::float8 AS confidence,
                review_state,
                properties,
                valid_from,
                valid_to,
                created_at,
                updated_at
            "#,
        )
        .bind(&edge_id)
        .bind(&edge.source_node_id)
        .bind(&edge.target_node_id)
        .bind(edge.relationship_type.as_str())
        .bind(edge.confidence)
        .bind(edge.review_state.as_str())
        .bind(&edge.properties)
        .bind(edge.valid_from)
        .bind(edge.valid_to)
        .fetch_one(&mut **transaction)
        .await?;

        for item in evidence {
            let evidence_id = evidence_id(&edge_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO graph_evidence (evidence_id, edge_id, source_kind, source_id, excerpt, metadata)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (edge_id, source_kind, source_id)
                DO UPDATE SET
                    excerpt = EXCLUDED.excerpt,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&edge_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(&item.excerpt)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        row_to_edge(row)
    }

    pub async fn summary(&self) -> Result<GraphSummary, GraphStoreError> {
        let node_count_rows = sqlx::query(
            r#"
            SELECT node_kind AS key, count(*) AS count
            FROM graph_nodes
            GROUP BY node_kind
            ORDER BY node_kind
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let node_counts = node_count_rows
            .into_iter()
            .map(row_to_count)
            .collect::<Result<Vec<_>, _>>()?;

        let edge_count_rows = sqlx::query(
            r#"
            SELECT relationship_type AS key, count(*) AS count
            FROM graph_edges
            GROUP BY relationship_type
            ORDER BY relationship_type
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        let edge_counts = edge_count_rows
            .into_iter()
            .map(row_to_count)
            .collect::<Result<Vec<_>, _>>()?;

        let evidence_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM graph_evidence")
            .fetch_one(&self.pool)
            .await?;
        let latest_projection_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT max(updated_at)
            FROM (
                SELECT updated_at FROM graph_nodes
                UNION ALL
                SELECT updated_at FROM graph_edges
            ) graph_updates
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        let total_nodes = node_counts.iter().map(|count| count.count).sum::<i64>();

        Ok(GraphSummary {
            node_counts,
            edge_counts,
            evidence_count,
            latest_projection_at,
            is_empty: total_nodes == 0,
        })
    }

    pub async fn search_nodes(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<GraphNode>, GraphStoreError> {
        let search_pattern = format!("%{query}%");
        let rows = sqlx::query(
            r#"
            SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
            FROM graph_nodes
            WHERE label ILIKE $1 OR stable_key ILIKE $1
            ORDER BY node_kind, label
            LIMIT $2
            "#,
        )
        .bind(search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_node).collect()
    }

    pub async fn neighborhood(
        &self,
        node_id: &str,
    ) -> Result<Option<GraphNeighborhood>, GraphStoreError> {
        let Some(selected_row) = sqlx::query(
            r#"
            SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
            FROM graph_nodes
            WHERE node_id = $1
            "#,
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };
        let selected_node = row_to_node(selected_row)?;

        let edge_rows = sqlx::query(
            r#"
            SELECT
                edge_id,
                source_node_id,
                target_node_id,
                relationship_type,
                confidence::float8 AS confidence,
                review_state,
                properties,
                valid_from,
                valid_to,
                created_at,
                updated_at
            FROM graph_edges
            WHERE valid_to IS NULL
              AND (source_node_id = $1 OR target_node_id = $1)
            ORDER BY relationship_type, source_node_id, target_node_id
            LIMIT $2
            "#,
        )
        .bind(&selected_node.node_id)
        .bind(GRAPH_NEIGHBORHOOD_EDGE_LIMIT)
        .fetch_all(&self.pool)
        .await?;
        let edges = edge_rows
            .into_iter()
            .map(row_to_edge)
            .collect::<Result<Vec<_>, _>>()?;

        let mut node_ids = BTreeSet::new();
        for edge in &edges {
            if edge.source_node_id != selected_node.node_id {
                node_ids.insert(edge.source_node_id.clone());
            }
            if edge.target_node_id != selected_node.node_id {
                node_ids.insert(edge.target_node_id.clone());
            }
        }
        let node_ids = node_ids.into_iter().collect::<Vec<_>>();
        let nodes = if node_ids.is_empty() {
            Vec::new()
        } else {
            let node_rows = sqlx::query(
                r#"
                SELECT node_id, node_kind, stable_key, label, properties, created_at, updated_at
                FROM graph_nodes
                WHERE node_id = ANY($1)
                ORDER BY node_kind, label, node_id
                "#,
            )
            .bind(&node_ids)
            .fetch_all(&self.pool)
            .await?;

            node_rows
                .into_iter()
                .map(row_to_node)
                .collect::<Result<Vec<_>, _>>()?
        };

        let edge_ids = edges
            .iter()
            .map(|edge| edge.edge_id.clone())
            .collect::<Vec<_>>();
        let evidence = if edge_ids.is_empty() {
            Vec::new()
        } else {
            let evidence_rows = sqlx::query(
                r#"
                SELECT edge_id, source_kind, source_id, excerpt, metadata
                FROM graph_evidence
                WHERE edge_id = ANY($1)
                ORDER BY edge_id, source_kind, source_id
                "#,
            )
            .bind(&edge_ids)
            .fetch_all(&self.pool)
            .await?;

            evidence_rows
                .into_iter()
                .map(row_to_evidence_summary)
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok(Some(GraphNeighborhood {
            selected_node,
            nodes,
            edges,
            evidence,
        }))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    Person,
    EmailAddress,
    Message,
    Document,
}

impl GraphNodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::EmailAddress => "email_address",
            Self::Message => "message",
            Self::Document => "document",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    PersonHasEmailAddress,
    PersonSentMessage,
    PersonReceivedMessage,
    EmailAddressSentMessage,
    EmailAddressReceivedMessage,
}

impl RelationshipType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersonHasEmailAddress => "person_has_email_address",
            Self::PersonSentMessage => "person_sent_message",
            Self::PersonReceivedMessage => "person_received_message",
            Self::EmailAddressSentMessage => "email_address_sent_message",
            Self::EmailAddressReceivedMessage => "email_address_received_message",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphReviewState {
    SystemAccepted,
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl GraphReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SystemAccepted => "system_accepted",
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEvidenceSourceKind {
    Contact,
    Message,
    Document,
    RawRecord,
}

impl GraphEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contact => "contact",
            Self::Message => "message",
            Self::Document => "document",
            Self::RawRecord => "raw_record",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewGraphNode {
    pub node_kind: GraphNodeKind,
    pub stable_key: String,
    pub label: String,
    pub properties: Value,
}

impl NewGraphNode {
    pub fn new(
        node_kind: GraphNodeKind,
        stable_key: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            node_kind,
            stable_key: stable_key.into(),
            label: label.into(),
            properties: json!({}),
        }
    }

    pub fn properties(mut self, properties: Value) -> Self {
        self.properties = properties;
        self
    }

    fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("stable_key", &self.stable_key)?;
        validate_non_empty("label", &self.label)?;
        validate_json_object("node properties", &self.properties)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewGraphEdge {
    pub source_node_id: String,
    pub target_node_id: String,
    pub relationship_type: RelationshipType,
    pub confidence: f64,
    pub review_state: GraphReviewState,
    pub properties: Value,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}

impl NewGraphEdge {
    pub fn new(
        source_node_id: String,
        target_node_id: String,
        relationship_type: RelationshipType,
        confidence: f64,
        review_state: GraphReviewState,
    ) -> Self {
        Self {
            source_node_id,
            target_node_id,
            relationship_type,
            confidence,
            review_state,
            properties: json!({}),
            valid_from: None,
            valid_to: None,
        }
    }

    pub fn properties(mut self, properties: Value) -> Self {
        self.properties = properties;
        self
    }

    fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("source_node_id", &self.source_node_id)?;
        validate_non_empty("target_node_id", &self.target_node_id)?;
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(GraphStoreError::InvalidConfidence(self.confidence));
        }
        if self.valid_to.is_some() {
            return Err(GraphStoreError::TemporalEdgesUnsupported);
        }
        validate_json_object("edge properties", &self.properties)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewGraphEvidence {
    pub source_kind: GraphEvidenceSourceKind,
    pub source_id: String,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

impl NewGraphEvidence {
    pub fn new(source_kind: GraphEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            excerpt: None,
            metadata: json!({}),
        }
    }

    pub fn excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_json_object("evidence metadata", &self.metadata)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphNode {
    pub node_id: String,
    pub node_kind: GraphNodeKind,
    pub stable_key: String,
    pub label: String,
    pub properties: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEdge {
    pub edge_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub relationship_type: RelationshipType,
    pub confidence: f64,
    pub review_state: GraphReviewState,
    pub properties: Value,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphCount {
    pub key: String,
    pub count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphSummary {
    pub node_counts: Vec<GraphCount>,
    pub edge_counts: Vec<GraphCount>,
    pub evidence_count: i64,
    pub latest_projection_at: Option<DateTime<Utc>>,
    pub is_empty: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphEvidenceSummary {
    pub edge_id: String,
    pub source_kind: GraphEvidenceSourceKind,
    pub source_id: String,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphNeighborhood {
    pub selected_node: GraphNode,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub evidence: Vec<GraphEvidenceSummary>,
}

#[derive(Debug, Error)]
pub enum GraphStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("graph edge confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(f64),

    #[error("graph edges require evidence in the first graph slice")]
    SystemEdgeRequiresEvidence,

    #[error("closed temporal graph edges are unsupported in the first graph slice")]
    TemporalEdgesUnsupported,

    #[error("unknown graph node kind stored in database: {0}")]
    UnknownNodeKind(String),

    #[error("unknown graph relationship type stored in database: {0}")]
    UnknownRelationshipType(String),

    #[error("unknown graph review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("unknown graph evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),
}

pub fn node_id(kind: GraphNodeKind, stable_key: &str) -> String {
    format!("graph:node:v1:{}:{stable_key}", kind.as_str())
}

pub fn edge_id(
    source_node_id: &str,
    relationship_type: RelationshipType,
    target_node_id: &str,
) -> String {
    format!(
        "graph:edge:v1:{}:{}:{}:{}:{}:{}",
        source_node_id.len(),
        source_node_id,
        relationship_type.as_str().len(),
        relationship_type.as_str(),
        target_node_id.len(),
        target_node_id
    )
}

pub fn evidence_id(edge_id: &str, source_kind: GraphEvidenceSourceKind, source_id: &str) -> String {
    format!(
        "graph:evidence:v1:{}:{}:{}:{}:{}:{}",
        edge_id.len(),
        edge_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}

fn row_to_node(row: PgRow) -> Result<GraphNode, GraphStoreError> {
    Ok(GraphNode {
        node_id: row.try_get("node_id")?,
        node_kind: parse_node_kind(row.try_get("node_kind")?)?,
        stable_key: row.try_get("stable_key")?,
        label: row.try_get("label")?,
        properties: row.try_get("properties")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_edge(row: PgRow) -> Result<GraphEdge, GraphStoreError> {
    Ok(GraphEdge {
        edge_id: row.try_get("edge_id")?,
        source_node_id: row.try_get("source_node_id")?,
        target_node_id: row.try_get("target_node_id")?,
        relationship_type: parse_relationship_type(row.try_get("relationship_type")?)?,
        confidence: row.try_get("confidence")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        properties: row.try_get("properties")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_count(row: PgRow) -> Result<GraphCount, GraphStoreError> {
    Ok(GraphCount {
        key: row.try_get("key")?,
        count: row.try_get("count")?,
    })
}

fn row_to_evidence_summary(row: PgRow) -> Result<GraphEvidenceSummary, GraphStoreError> {
    Ok(GraphEvidenceSummary {
        edge_id: row.try_get("edge_id")?,
        source_kind: parse_evidence_source_kind(row.try_get("source_kind")?)?,
        source_id: row.try_get("source_id")?,
        excerpt: row.try_get("excerpt")?,
        metadata: row.try_get("metadata")?,
    })
}

fn parse_node_kind(value: String) -> Result<GraphNodeKind, GraphStoreError> {
    match value.as_str() {
        "person" => Ok(GraphNodeKind::Person),
        "email_address" => Ok(GraphNodeKind::EmailAddress),
        "message" => Ok(GraphNodeKind::Message),
        "document" => Ok(GraphNodeKind::Document),
        _ => Err(GraphStoreError::UnknownNodeKind(value)),
    }
}

fn parse_relationship_type(value: String) -> Result<RelationshipType, GraphStoreError> {
    match value.as_str() {
        "person_has_email_address" => Ok(RelationshipType::PersonHasEmailAddress),
        "person_sent_message" => Ok(RelationshipType::PersonSentMessage),
        "person_received_message" => Ok(RelationshipType::PersonReceivedMessage),
        "email_address_sent_message" => Ok(RelationshipType::EmailAddressSentMessage),
        "email_address_received_message" => Ok(RelationshipType::EmailAddressReceivedMessage),
        _ => Err(GraphStoreError::UnknownRelationshipType(value)),
    }
}

fn parse_review_state(value: String) -> Result<GraphReviewState, GraphStoreError> {
    match value.as_str() {
        "system_accepted" => Ok(GraphReviewState::SystemAccepted),
        "suggested" => Ok(GraphReviewState::Suggested),
        "user_confirmed" => Ok(GraphReviewState::UserConfirmed),
        "user_rejected" => Ok(GraphReviewState::UserRejected),
        _ => Err(GraphStoreError::UnknownReviewState(value)),
    }
}

fn parse_evidence_source_kind(value: String) -> Result<GraphEvidenceSourceKind, GraphStoreError> {
    match value.as_str() {
        "contact" => Ok(GraphEvidenceSourceKind::Contact),
        "message" => Ok(GraphEvidenceSourceKind::Message),
        "document" => Ok(GraphEvidenceSourceKind::Document),
        "raw_record" => Ok(GraphEvidenceSourceKind::RawRecord),
        _ => Err(GraphStoreError::UnknownEvidenceSourceKind(value)),
    }
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), GraphStoreError> {
    if value.trim().is_empty() {
        return Err(GraphStoreError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_edge_with_evidence(
    edge: &NewGraphEdge,
    evidence: &[NewGraphEvidence],
) -> Result<(), GraphStoreError> {
    edge.validate()?;
    if evidence.is_empty() {
        return Err(GraphStoreError::SystemEdgeRequiresEvidence);
    }
    for item in evidence {
        item.validate()?;
    }

    Ok(())
}

fn validate_json_object(field_name: &'static str, value: &Value) -> Result<(), GraphStoreError> {
    if !value.is_object() {
        return Err(GraphStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
