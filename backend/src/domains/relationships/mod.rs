use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, GraphStoreError,
    NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType as GraphRelationshipType,
    node_id,
};

pub mod api;

#[derive(Clone)]
pub struct RelationshipStore {
    pool: PgPool,
}

impl RelationshipStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipStoreError> {
        validate_relationship_with_evidence(relationship, evidence)?;

        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::upsert_with_evidence_in_transaction(&mut transaction, relationship, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relationship: &NewRelationship,
        evidence: &[NewRelationshipEvidence],
    ) -> Result<Relationship, RelationshipStoreError> {
        let relationship_id = relationship_id(
            relationship.source_entity_kind,
            &relationship.source_entity_id,
            &relationship.relationship_type,
            relationship.target_entity_kind,
            &relationship.target_entity_id,
        );
        let row = sqlx::query(
            r#"
            INSERT INTO relationships (
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score,
                strength_score,
                confidence,
                review_state,
                valid_from,
                valid_to,
                metadata
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                CAST($7 AS NUMERIC(5,4)),
                CAST($8 AS NUMERIC(5,4)),
                CAST($9 AS NUMERIC(5,4)),
                $10,
                $11,
                $12,
                $13
            )
            ON CONFLICT (relationship_id)
            DO UPDATE SET
                trust_score = EXCLUDED.trust_score,
                strength_score = EXCLUDED.strength_score,
                confidence = EXCLUDED.confidence,
                review_state = EXCLUDED.review_state,
                valid_from = EXCLUDED.valid_from,
                valid_to = EXCLUDED.valid_to,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&relationship_id)
        .bind(relationship.source_entity_kind.as_str())
        .bind(&relationship.source_entity_id)
        .bind(relationship.target_entity_kind.as_str())
        .bind(&relationship.target_entity_id)
        .bind(&relationship.relationship_type)
        .bind(relationship.trust_score)
        .bind(relationship.strength_score)
        .bind(relationship.confidence)
        .bind(relationship.review_state.as_str())
        .bind(relationship.valid_from)
        .bind(relationship.valid_to)
        .bind(&relationship.metadata)
        .fetch_one(&mut **transaction)
        .await?;
        let stored = row_to_relationship(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&relationship_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO relationship_evidence (
                    evidence_id,
                    relationship_id,
                    source_kind,
                    source_id,
                    excerpt,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (relationship_id, source_kind, source_id)
                DO UPDATE SET
                    excerpt = EXCLUDED.excerpt,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&relationship_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(&item.excerpt)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        Self::project_relationship_graph_in_transaction(transaction, &stored).await?;

        Ok(stored)
    }

    async fn project_relationship_graph_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relationship: &Relationship,
    ) -> Result<(), RelationshipStoreError> {
        let Some(projection) = relationship_graph_projection(relationship) else {
            return Ok(());
        };

        let source_node = relationship_graph_node_in_transaction(
            transaction,
            relationship.source_entity_kind,
            &relationship.source_entity_id,
        )
        .await?;
        let target_node = relationship_graph_node_in_transaction(
            transaction,
            relationship.target_entity_kind,
            &relationship.target_entity_id,
        )
        .await?;

        GraphStore::upsert_node_in_transaction(transaction, &source_node).await?;
        GraphStore::upsert_node_in_transaction(transaction, &target_node).await?;
        GraphStore::upsert_edge_with_evidence_in_transaction(
            transaction,
            &projection.edge,
            &[projection.evidence],
        )
        .await?;

        Ok(())
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: RelationshipEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Relationship>, RelationshipStoreError> {
        validate_non_empty("entity_id", entity_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            FROM relationships
            WHERE (source_entity_kind = $1 AND source_entity_id = $2)
               OR (target_entity_kind = $1 AND target_entity_id = $2)
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_relationship).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: RelationshipReviewState,
        limit: i64,
    ) -> Result<Vec<Relationship>, RelationshipStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            FROM relationships
            WHERE review_state = $1
            ORDER BY updated_at DESC, relationship_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_relationship).collect()
    }

    pub async fn set_review_state(
        &self,
        relationship_id: &str,
        review_state: RelationshipReviewState,
    ) -> Result<Relationship, RelationshipStoreError> {
        validate_non_empty("relationship_id", relationship_id)?;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE relationships
            SET
                review_state = $1,
                updated_at = now()
            WHERE relationship_id = $2
            RETURNING
                relationship_id,
                source_entity_kind,
                source_entity_id,
                target_entity_kind,
                target_entity_id,
                relationship_type,
                trust_score::float8 AS trust_score,
                strength_score::float8 AS strength_score,
                confidence::float8 AS confidence,
                review_state,
                valid_from,
                valid_to,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(relationship_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(RelationshipStoreError::RelationshipNotFound)?;

        let relationship = row_to_relationship(row)?;
        Self::project_relationship_graph_in_transaction(&mut transaction, &relationship).await?;
        transaction.commit().await?;

        Ok(relationship)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipEntityKind {
    Persona,
    Organization,
    Project,
    Communication,
    Document,
    Task,
    Event,
    Decision,
    Obligation,
    Knowledge,
}

impl RelationshipEntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Persona => "persona",
            Self::Organization => "organization",
            Self::Project => "project",
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Task => "task",
            Self::Event => "event",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Knowledge => "knowledge",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, RelationshipStoreError> {
        let value = value.as_ref().trim();
        match value {
            "persona" => Ok(Self::Persona),
            "organization" => Ok(Self::Organization),
            "project" => Ok(Self::Project),
            "communication" => Ok(Self::Communication),
            "document" => Ok(Self::Document),
            "task" => Ok(Self::Task),
            "event" => Ok(Self::Event),
            "decision" => Ok(Self::Decision),
            "obligation" => Ok(Self::Obligation),
            "knowledge" => Ok(Self::Knowledge),
            _ => Err(RelationshipStoreError::UnknownEntityKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipEvidenceSourceKind {
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Project,
    Organization,
    Persona,
    RawRecord,
}

impl RelationshipEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Event => "event",
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Task => "task",
            Self::Project => "project",
            Self::Organization => "organization",
            Self::Persona => "persona",
            Self::RawRecord => "raw_record",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipReviewState {
    Suggested,
    SystemAccepted,
    UserConfirmed,
    UserRejected,
}

impl RelationshipReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::SystemAccepted => "system_accepted",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, RelationshipStoreError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "system_accepted" => Ok(Self::SystemAccepted),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(RelationshipStoreError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewRelationship {
    pub source_entity_kind: RelationshipEntityKind,
    pub source_entity_id: String,
    pub target_entity_kind: RelationshipEntityKind,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub trust_score: f64,
    pub strength_score: f64,
    pub confidence: f64,
    pub review_state: RelationshipReviewState,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewRelationship {
    pub fn between_personas(
        source_persona_id: impl Into<String>,
        target_persona_id: impl Into<String>,
        relationship_type: impl Into<String>,
        trust_score: f64,
        strength_score: f64,
        confidence: f64,
        review_state: RelationshipReviewState,
    ) -> Self {
        Self {
            source_entity_kind: RelationshipEntityKind::Persona,
            source_entity_id: source_persona_id.into(),
            target_entity_kind: RelationshipEntityKind::Persona,
            target_entity_id: target_persona_id.into(),
            relationship_type: relationship_type.into(),
            trust_score,
            strength_score,
            confidence,
            review_state,
            valid_from: None,
            valid_to: None,
            metadata: json!({}),
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    fn validate(&self) -> Result<(), RelationshipStoreError> {
        validate_non_empty("source_entity_id", &self.source_entity_id)?;
        validate_non_empty("target_entity_id", &self.target_entity_id)?;
        validate_non_empty("relationship_type", &self.relationship_type)?;
        validate_score("trust_score", self.trust_score)?;
        validate_score("strength_score", self.strength_score)?;
        validate_score("confidence", self.confidence)?;
        validate_json_object("relationship metadata", &self.metadata)?;
        if self.source_entity_kind == self.target_entity_kind
            && self.source_entity_id == self.target_entity_id
        {
            return Err(RelationshipStoreError::IdenticalEndpoints);
        }
        if let (Some(valid_from), Some(valid_to)) = (self.valid_from, self.valid_to) {
            if valid_to < valid_from {
                return Err(RelationshipStoreError::InvalidTemporalRange);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewRelationshipEvidence {
    pub source_kind: RelationshipEvidenceSourceKind,
    pub source_id: String,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

impl NewRelationshipEvidence {
    pub fn new(source_kind: RelationshipEvidenceSourceKind, source_id: impl Into<String>) -> Self {
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

    fn validate(&self) -> Result<(), RelationshipStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_json_object("evidence metadata", &self.metadata)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Relationship {
    pub relationship_id: String,
    pub source_entity_kind: RelationshipEntityKind,
    pub source_entity_id: String,
    pub target_entity_kind: RelationshipEntityKind,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub trust_score: f64,
    pub strength_score: f64,
    pub confidence: f64,
    pub review_state: RelationshipReviewState,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum RelationshipStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] GraphStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("relationship evidence is required")]
    MissingEvidence,

    #[error("relationship was not found")]
    RelationshipNotFound,

    #[error("relationship endpoints must be distinct")]
    IdenticalEndpoints,

    #[error("relationship valid_to must not be earlier than valid_from")]
    InvalidTemporalRange,

    #[error("unknown relationship entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown relationship evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown relationship review state stored in database: {0}")]
    UnknownReviewState(String),
}

pub fn relationship_id(
    source_entity_kind: RelationshipEntityKind,
    source_entity_id: &str,
    relationship_type: &str,
    target_entity_kind: RelationshipEntityKind,
    target_entity_id: &str,
) -> String {
    format!(
        "relationship:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        source_entity_kind.as_str().len(),
        source_entity_kind.as_str(),
        source_entity_id.len(),
        source_entity_id,
        relationship_type.len(),
        relationship_type,
        target_entity_kind.as_str().len(),
        target_entity_kind.as_str(),
        target_entity_id.len(),
        target_entity_id
    )
}

pub fn evidence_id(
    relationship_id: &str,
    source_kind: RelationshipEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "relationship:evidence:v1:{}:{}:{}:{}:{}:{}",
        relationship_id.len(),
        relationship_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}

fn validate_relationship_with_evidence(
    relationship: &NewRelationship,
    evidence: &[NewRelationshipEvidence],
) -> Result<(), RelationshipStoreError> {
    relationship.validate()?;
    if evidence.is_empty() {
        return Err(RelationshipStoreError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }

    Ok(())
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), RelationshipStoreError> {
    if value.trim().is_empty() {
        return Err(RelationshipStoreError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_score(field_name: &'static str, value: f64) -> Result<(), RelationshipStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(RelationshipStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), RelationshipStoreError> {
    if !value.is_object() {
        return Err(RelationshipStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}

fn row_to_relationship(row: PgRow) -> Result<Relationship, RelationshipStoreError> {
    Ok(Relationship {
        relationship_id: row.try_get("relationship_id")?,
        source_entity_kind: parse_entity_kind(row.try_get("source_entity_kind")?)?,
        source_entity_id: row.try_get("source_entity_id")?,
        target_entity_kind: parse_entity_kind(row.try_get("target_entity_kind")?)?,
        target_entity_id: row.try_get("target_entity_id")?,
        relationship_type: row.try_get("relationship_type")?,
        trust_score: row.try_get("trust_score")?,
        strength_score: row.try_get("strength_score")?,
        confidence: row.try_get("confidence")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<RelationshipEntityKind, RelationshipStoreError> {
    RelationshipEntityKind::parse(value)
}

fn parse_review_state(value: String) -> Result<RelationshipReviewState, RelationshipStoreError> {
    RelationshipReviewState::parse(value)
}

struct RelationshipGraphProjection {
    edge: NewGraphEdge,
    evidence: NewGraphEvidence,
}

fn relationship_graph_projection(
    relationship: &Relationship,
) -> Option<RelationshipGraphProjection> {
    if relationship.valid_to.is_some() {
        return None;
    }
    let source_node_kind = relationship_graph_node_kind(relationship.source_entity_kind)?;
    let target_node_kind = relationship_graph_node_kind(relationship.target_entity_kind)?;

    let edge = NewGraphEdge::new(
        node_id(source_node_kind, &relationship.source_entity_id),
        node_id(target_node_kind, &relationship.target_entity_id),
        GraphRelationshipType::EntityRelationship,
        relationship.confidence,
        graph_review_state(relationship.review_state),
    )
    .properties(json!({
        "source": "relationships",
        "relationship_id": relationship.relationship_id,
        "relationship_type": relationship.relationship_type,
        "source_entity_kind": relationship.source_entity_kind.as_str(),
        "source_entity_id": relationship.source_entity_id,
        "target_entity_kind": relationship.target_entity_kind.as_str(),
        "target_entity_id": relationship.target_entity_id,
        "trust_score": relationship.trust_score,
        "strength_score": relationship.strength_score,
    }));

    let evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Relationship,
        &relationship.relationship_id,
    )
    .excerpt(relationship.relationship_type.clone())
    .metadata(json!({
        "projection": "relationship_graph",
        "trust_score": relationship.trust_score,
        "strength_score": relationship.strength_score,
        "review_state": relationship.review_state.as_str(),
    }));

    Some(RelationshipGraphProjection { edge, evidence })
}

fn relationship_graph_node_kind(entity_kind: RelationshipEntityKind) -> Option<GraphNodeKind> {
    match entity_kind {
        RelationshipEntityKind::Persona => Some(GraphNodeKind::Person),
        RelationshipEntityKind::Organization => Some(GraphNodeKind::Organization),
        RelationshipEntityKind::Communication => Some(GraphNodeKind::Message),
        RelationshipEntityKind::Document => Some(GraphNodeKind::Document),
        RelationshipEntityKind::Project => Some(GraphNodeKind::Project),
        RelationshipEntityKind::Task => Some(GraphNodeKind::Task),
        RelationshipEntityKind::Event => Some(GraphNodeKind::Event),
        RelationshipEntityKind::Decision => Some(GraphNodeKind::Decision),
        RelationshipEntityKind::Obligation => Some(GraphNodeKind::Obligation),
        RelationshipEntityKind::Knowledge => Some(GraphNodeKind::Knowledge),
    }
}

async fn relationship_graph_node_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    entity_kind: RelationshipEntityKind,
    entity_id: &str,
) -> Result<NewGraphNode, RelationshipStoreError> {
    if entity_kind == RelationshipEntityKind::Persona {
        return persona_graph_node_in_transaction(transaction, entity_id).await;
    }

    let graph_node_kind =
        relationship_graph_node_kind(entity_kind).expect("projection must use supported kind");
    Ok(
        NewGraphNode::new(graph_node_kind, entity_id, entity_id).properties(json!({
            "entity_kind": entity_kind.as_str(),
            "entity_id": entity_id,
        })),
    )
}

async fn persona_graph_node_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: &str,
) -> Result<NewGraphNode, RelationshipStoreError> {
    let row = sqlx::query("SELECT display_name, email_address FROM persons WHERE person_id = $1")
        .bind(persona_id)
        .fetch_optional(&mut **transaction)
        .await?;

    if let Some(row) = row {
        let display_name: String = row.try_get("display_name")?;
        let email_address: String = row.try_get("email_address")?;
        return Ok(
            NewGraphNode::new(GraphNodeKind::Person, persona_id, display_name).properties(json!({
                "persona_id": persona_id,
                "email_address": email_address,
            })),
        );
    }

    Ok(
        NewGraphNode::new(GraphNodeKind::Person, persona_id, persona_id)
            .properties(json!({ "persona_id": persona_id })),
    )
}

fn graph_review_state(review_state: RelationshipReviewState) -> GraphReviewState {
    match review_state {
        RelationshipReviewState::Suggested => GraphReviewState::Suggested,
        RelationshipReviewState::SystemAccepted => GraphReviewState::SystemAccepted,
        RelationshipReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        RelationshipReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::graph::core::{
        GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState,
        RelationshipType as GraphRelationshipType, node_id,
    };

    #[test]
    fn persona_relationship_graph_projection_preserves_domain_relationship_semantics() {
        let now = Utc::now();
        let relationship = Relationship {
            relationship_id: "relationship:v1:test".to_owned(),
            source_entity_kind: RelationshipEntityKind::Persona,
            source_entity_id: "person:v1:email:source@example.com".to_owned(),
            target_entity_kind: RelationshipEntityKind::Persona,
            target_entity_id: "person:v1:email:target@example.com".to_owned(),
            relationship_type: "knows".to_owned(),
            trust_score: 0.77,
            strength_score: 0.58,
            confidence: 0.83,
            review_state: RelationshipReviewState::Suggested,
            valid_from: None,
            valid_to: None,
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };

        let projection = relationship_graph_projection(&relationship)
            .expect("active Persona relationship must have graph projection");

        assert_eq!(
            projection.edge.source_node_id,
            node_id(GraphNodeKind::Person, &relationship.source_entity_id)
        );
        assert_eq!(
            projection.edge.target_node_id,
            node_id(GraphNodeKind::Person, &relationship.target_entity_id)
        );
        assert_eq!(
            projection.edge.relationship_type,
            GraphRelationshipType::EntityRelationship
        );
        assert_eq!(projection.edge.confidence, relationship.confidence);
        assert_eq!(projection.edge.review_state, GraphReviewState::Suggested);
        assert_eq!(
            projection.edge.properties["relationship_id"],
            json!(relationship.relationship_id)
        );
        assert_eq!(
            projection.edge.properties["relationship_type"],
            json!(relationship.relationship_type)
        );
        assert_eq!(
            projection.edge.properties["trust_score"],
            json!(relationship.trust_score)
        );
        assert_eq!(
            projection.edge.properties["strength_score"],
            json!(relationship.strength_score)
        );
        assert_eq!(
            projection.evidence.source_kind,
            GraphEvidenceSourceKind::Relationship
        );
        assert_eq!(projection.evidence.source_id, relationship.relationship_id);
    }
}
