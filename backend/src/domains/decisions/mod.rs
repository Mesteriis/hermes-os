use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::graph::core::{
    GraphEvidenceSourceKind, GraphNodeKind, GraphReviewState, GraphStore, GraphStoreError,
    NewGraphEdge, NewGraphEvidence, NewGraphNode, RelationshipType as GraphRelationshipType,
};
use crate::engines::decision::{
    DecisionEngine, DecisionEngineError, DecisionExtractionInput, DecisionExtractionResult,
};

pub mod api;

const MAX_REFRESH_LIMIT: i64 = 100;
const MIN_REFRESH_LIMIT: i64 = 1;

#[derive(Clone)]
pub struct DecisionStore {
    pool: PgPool,
}

impl DecisionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        decision: &NewDecision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<Decision, DecisionStoreError> {
        validate_decision_with_evidence(decision, evidence, impacted_entities)?;

        let mut transaction = self.pool.begin().await?;
        let stored = Self::upsert_with_evidence_in_transaction(
            &mut transaction,
            decision,
            evidence,
            impacted_entities,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn refresh_deterministic_candidates(
        &self,
        limit: i64,
    ) -> Result<usize, DecisionStoreError> {
        let limit = validate_refresh_limit(limit)?;
        let message_count = self.refresh_message_candidates(limit).await?;
        let document_count = self.refresh_document_candidates(limit).await?;

        Ok(message_count + document_count)
    }

    pub async fn refresh_message_candidates_for_ids(
        &self,
        message_ids: &[String],
    ) -> Result<usize, DecisionStoreError> {
        if message_ids.is_empty() {
            return Ok(0);
        }

        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                subject,
                body_text
            FROM communication_messages
            WHERE message_id = ANY($1)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            "#,
        )
        .bind(message_ids.to_vec())
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("message_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("subject")?,
                row.try_get::<String, _>("body_text")?,
            );
            count += self
                .refresh_communication_decision_candidates(&source_id, &source_text)
                .await?;
        }

        Ok(count)
    }

    async fn refresh_message_candidates(&self, limit: i64) -> Result<usize, DecisionStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                subject,
                body_text
            FROM communication_messages
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("message_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("subject")?,
                row.try_get::<String, _>("body_text")?,
            );
            count += self
                .refresh_communication_decision_candidates(&source_id, &source_text)
                .await?;
        }

        Ok(count)
    }

    async fn refresh_communication_decision_candidates(
        &self,
        source_id: &str,
        source_text: &str,
    ) -> Result<usize, DecisionStoreError> {
        let input = DecisionExtractionInput::communication(
            source_id,
            source_text,
            DecisionEntityKind::Communication,
            source_id,
        );
        let extraction = DecisionEngine::detect_candidates(&input)?;
        self.persist_decision_extraction(extraction).await
    }

    async fn refresh_document_candidates(&self, limit: i64) -> Result<usize, DecisionStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                document_id,
                title,
                extracted_text
            FROM documents
            ORDER BY imported_at DESC, document_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0usize;
        for row in rows {
            let source_id = row.try_get::<String, _>("document_id")?;
            let source_text = format!(
                "{}\n{}",
                row.try_get::<String, _>("title")?,
                row.try_get::<String, _>("extracted_text")?,
            );
            let input = DecisionExtractionInput::document(
                &source_id,
                &source_text,
                DecisionEntityKind::Document,
                &source_id,
            );
            let extraction = DecisionEngine::detect_candidates(&input)?;
            count += self.persist_decision_extraction(extraction).await?;
        }

        Ok(count)
    }

    async fn persist_decision_extraction(
        &self,
        extraction: DecisionExtractionResult,
    ) -> Result<usize, DecisionStoreError> {
        let mut count = 0usize;
        for candidate in extraction.decisions {
            let (mut decision, evidence, impacted_entities) = candidate.to_decision_draft();
            preserve_existing_review_state(&self.pool, &mut decision).await?;
            self.upsert_with_evidence(&decision, &[evidence], &impacted_entities)
                .await?;
            count += 1;
        }

        Ok(count)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        decision: &NewDecision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<Decision, DecisionStoreError> {
        let decision_id = decision_id(decision);
        let row = sqlx::query(
            r#"
            INSERT INTO decisions (
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence,
                metadata
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                CAST($10 AS NUMERIC(5,4)),
                $11
            )
            ON CONFLICT (decision_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                status = EXCLUDED.status,
                rationale = EXCLUDED.rationale,
                alternatives = EXCLUDED.alternatives,
                decided_by_entity_kind = EXCLUDED.decided_by_entity_kind,
                decided_by_entity_id = EXCLUDED.decided_by_entity_id,
                decided_at = EXCLUDED.decided_at,
                review_state = EXCLUDED.review_state,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&decision_id)
        .bind(&decision.title)
        .bind(decision.status.as_str())
        .bind(&decision.rationale)
        .bind(&decision.alternatives)
        .bind(decision.decided_by_entity_kind.map(|kind| kind.as_str()))
        .bind(&decision.decided_by_entity_id)
        .bind(decision.decided_at)
        .bind(decision.review_state.as_str())
        .bind(decision.confidence)
        .bind(&decision.metadata)
        .fetch_one(&mut **transaction)
        .await?;

        let stored = row_to_decision(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&decision_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO decision_evidence (
                    evidence_id,
                    decision_id,
                    source_kind,
                    source_id,
                    quote,
                    confidence,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, CAST($6 AS NUMERIC(5,4)), $7)
                ON CONFLICT (decision_id, source_kind, source_id)
                DO UPDATE SET
                    quote = EXCLUDED.quote,
                    confidence = EXCLUDED.confidence,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&decision_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(&item.quote)
            .bind(item.confidence)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        for item in impacted_entities {
            sqlx::query(
                r#"
                INSERT INTO decision_impacted_entities (
                    decision_id,
                    entity_kind,
                    entity_id,
                    impact_type,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (decision_id, entity_kind, entity_id)
                DO UPDATE SET
                    impact_type = EXCLUDED.impact_type,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(&decision_id)
            .bind(item.entity_kind.as_str())
            .bind(&item.entity_id)
            .bind(&item.impact_type)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        Self::project_decision_graph_in_transaction(
            transaction,
            &stored,
            evidence,
            impacted_entities,
        )
        .await?;

        Ok(stored)
    }

    async fn project_decision_graph_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        decision: &Decision,
        evidence: &[NewDecisionEvidence],
        impacted_entities: &[NewDecisionImpactedEntity],
    ) -> Result<(), DecisionStoreError> {
        if impacted_entities.is_empty() {
            return Ok(());
        }

        let decision_node = NewGraphNode::new(
            GraphNodeKind::Decision,
            decision.decision_id.clone(),
            decision.title.clone(),
        )
        .properties(json!({
            "domain": "decision",
            "decision_id": decision.decision_id,
            "status": decision.status.as_str(),
            "review_state": decision.review_state.as_str(),
        }));
        let stored_decision_node =
            GraphStore::upsert_node_in_transaction(transaction, &decision_node).await?;

        for impacted_entity in impacted_entities {
            let Some(target_node_kind) =
                decision_entity_to_graph_node_kind(impacted_entity.entity_kind)
            else {
                continue;
            };
            let target_node = NewGraphNode::new(
                target_node_kind,
                impacted_entity.entity_id.clone(),
                impacted_entity.entity_id.clone(),
            )
            .properties(json!({
                "domain": impacted_entity.entity_kind.as_str(),
                "entity_id": impacted_entity.entity_id,
            }));
            let stored_target_node =
                GraphStore::upsert_node_in_transaction(transaction, &target_node).await?;

            let graph_edge = NewGraphEdge::new(
                stored_decision_node.node_id.clone(),
                stored_target_node.node_id,
                GraphRelationshipType::EntityRelationship,
                decision.confidence,
                decision_review_state_to_graph_review_state(decision.review_state),
            )
            .properties(json!({
                "domain": "decision",
                "decision_id": decision.decision_id,
                "impact_type": impacted_entity.impact_type,
            }));
            let graph_evidence = decision_graph_evidence(decision, evidence);

            GraphStore::upsert_edge_with_evidence_in_transaction(
                transaction,
                &graph_edge,
                &[graph_evidence],
            )
            .await?;
        }

        Ok(())
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: DecisionEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Decision>, DecisionStoreError> {
        validate_non_empty("entity_id", entity_id)?;
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                decision.decision_id,
                decision.title,
                decision.status,
                decision.rationale,
                decision.alternatives,
                decision.decided_by_entity_kind,
                decision.decided_by_entity_id,
                decision.decided_at,
                decision.review_state,
                decision.confidence::float8 AS confidence,
                decision.metadata,
                decision.created_at,
                decision.updated_at
            FROM decisions decision
            LEFT JOIN decision_impacted_entities impacted
              ON impacted.decision_id = decision.decision_id
            WHERE (decision.decided_by_entity_kind = $1 AND decision.decided_by_entity_id = $2)
               OR (impacted.entity_kind = $1 AND impacted.entity_id = $2)
            ORDER BY decision.updated_at DESC, decision.decision_id ASC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_decision).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: DecisionReviewState,
        limit: i64,
    ) -> Result<Vec<Decision>, DecisionStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM decisions
            WHERE review_state = $1
            ORDER BY updated_at DESC, decision_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_decision).collect()
    }

    pub async fn set_review_state(
        &self,
        decision_id: &str,
        review_state: DecisionReviewState,
    ) -> Result<Decision, DecisionStoreError> {
        validate_non_empty("decision_id", decision_id)?;
        let row = sqlx::query(
            r#"
            UPDATE decisions
            SET
                review_state = $1,
                updated_at = now()
            WHERE decision_id = $2
            RETURNING
                decision_id,
                title,
                status,
                rationale,
                alternatives,
                decided_by_entity_kind,
                decided_by_entity_id,
                decided_at,
                review_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(decision_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DecisionStoreError::DecisionNotFound)?;

        row_to_decision(row)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionEntityKind {
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

impl DecisionEntityKind {
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

    pub fn parse(value: impl AsRef<str>) -> Result<Self, DecisionStoreError> {
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
            _ => Err(DecisionStoreError::UnknownEntityKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionEvidenceSourceKind {
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Relationship,
    Project,
    Organization,
    Persona,
    RawRecord,
}

impl DecisionEvidenceSourceKind {
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
            Self::Relationship => "relationship",
            Self::Project => "project",
            Self::Organization => "organization",
            Self::Persona => "persona",
            Self::RawRecord => "raw_record",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Active,
    Superseded,
    Reversed,
    Deprecated,
}

impl DecisionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Superseded => "superseded",
            Self::Reversed => "reversed",
            Self::Deprecated => "deprecated",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl DecisionReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, DecisionStoreError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(DecisionStoreError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewDecision {
    pub title: String,
    pub status: DecisionStatus,
    pub rationale: String,
    pub alternatives: Value,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub review_state: DecisionReviewState,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewDecision {
    pub fn new(
        title: impl Into<String>,
        rationale: impl Into<String>,
        confidence: f64,
        review_state: DecisionReviewState,
    ) -> Self {
        Self {
            title: title.into(),
            status: DecisionStatus::Active,
            rationale: rationale.into(),
            alternatives: json!([]),
            decided_by_entity_kind: None,
            decided_by_entity_id: None,
            decided_at: None,
            review_state,
            confidence,
            metadata: json!({}),
        }
    }

    pub fn status(mut self, status: DecisionStatus) -> Self {
        self.status = status;
        self
    }

    pub fn alternatives(mut self, alternatives: Value) -> Self {
        self.alternatives = alternatives;
        self
    }

    pub fn decided_by(
        mut self,
        decided_by_entity_kind: DecisionEntityKind,
        decided_by_entity_id: impl Into<String>,
    ) -> Self {
        self.decided_by_entity_kind = Some(decided_by_entity_kind);
        self.decided_by_entity_id = Some(decided_by_entity_id.into());
        self
    }

    pub fn decided_at(mut self, decided_at: DateTime<Utc>) -> Self {
        self.decided_at = Some(decided_at);
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    fn validate(&self) -> Result<(), DecisionStoreError> {
        validate_non_empty("title", &self.title)?;
        validate_non_empty("rationale", &self.rationale)?;
        validate_score("confidence", self.confidence)?;
        validate_json_array("alternatives", &self.alternatives)?;
        validate_json_object("decision metadata", &self.metadata)?;

        match (
            self.decided_by_entity_kind,
            self.decided_by_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(decided_by_entity_id)) => {
                validate_non_empty("decided_by_entity_id", decided_by_entity_id)?;
            }
            _ => return Err(DecisionStoreError::PartialDecider),
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewDecisionEvidence {
    pub source_kind: DecisionEvidenceSourceKind,
    pub source_id: String,
    pub quote: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewDecisionEvidence {
    pub fn new(source_kind: DecisionEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            quote: None,
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn quote(mut self, quote: impl Into<String>) -> Self {
        self.quote = Some(quote.into());
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    fn validate(&self) -> Result<(), DecisionStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_score("evidence confidence", self.confidence)?;
        validate_json_object("evidence metadata", &self.metadata)?;
        if let Some(quote) = &self.quote {
            validate_non_empty("quote", quote)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewDecisionImpactedEntity {
    pub entity_kind: DecisionEntityKind,
    pub entity_id: String,
    pub impact_type: String,
    pub metadata: Value,
}

impl NewDecisionImpactedEntity {
    pub fn new(entity_kind: DecisionEntityKind, entity_id: impl Into<String>) -> Self {
        Self {
            entity_kind,
            entity_id: entity_id.into(),
            impact_type: "related".to_owned(),
            metadata: json!({}),
        }
    }

    pub fn impact_type(mut self, impact_type: impl Into<String>) -> Self {
        self.impact_type = impact_type.into();
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    fn validate(&self) -> Result<(), DecisionStoreError> {
        validate_non_empty("entity_id", &self.entity_id)?;
        validate_non_empty("impact_type", &self.impact_type)?;
        validate_json_object("impact metadata", &self.metadata)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Decision {
    pub decision_id: String,
    pub title: String,
    pub status: DecisionStatus,
    pub rationale: String,
    pub alternatives: Value,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub review_state: DecisionReviewState,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum DecisionStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Graph(#[from] GraphStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be a JSON array")]
    InvalidJsonArray(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("decision evidence is required")]
    MissingEvidence,

    #[error("decision was not found")]
    DecisionNotFound,

    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("decided_by entity kind and id must be provided together")]
    PartialDecider,

    #[error("unknown decision entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown decision evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown decision status stored in database: {0}")]
    UnknownStatus(String),

    #[error("unknown decision review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error(transparent)]
    DecisionEngine(#[from] DecisionEngineError),
}

pub fn decision_id(decision: &NewDecision) -> String {
    let title = normalize_text(&decision.title);
    let decider_kind = decision
        .decided_by_entity_kind
        .map(DecisionEntityKind::as_str)
        .unwrap_or("");
    let decider_id = decision.decided_by_entity_id.as_deref().unwrap_or("");
    let decided_at = decision
        .decided_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_default();

    format!(
        "decision:v1:{}:{}:{}:{}:{}:{}:{}:{}",
        title.len(),
        title,
        decider_kind.len(),
        decider_kind,
        decider_id.len(),
        decider_id,
        decided_at.len(),
        decided_at
    )
}

pub fn evidence_id(
    decision_id: &str,
    source_kind: DecisionEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "decision:evidence:v1:{}:{}:{}:{}:{}:{}",
        decision_id.len(),
        decision_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}

fn decision_entity_to_graph_node_kind(entity_kind: DecisionEntityKind) -> Option<GraphNodeKind> {
    match entity_kind {
        DecisionEntityKind::Persona => Some(GraphNodeKind::Person),
        DecisionEntityKind::Project => Some(GraphNodeKind::Project),
        DecisionEntityKind::Communication => Some(GraphNodeKind::Message),
        DecisionEntityKind::Document => Some(GraphNodeKind::Document),
        DecisionEntityKind::Decision => Some(GraphNodeKind::Decision),
        DecisionEntityKind::Organization
        | DecisionEntityKind::Task
        | DecisionEntityKind::Event
        | DecisionEntityKind::Obligation
        | DecisionEntityKind::Knowledge => None,
    }
}

fn decision_review_state_to_graph_review_state(
    review_state: DecisionReviewState,
) -> GraphReviewState {
    match review_state {
        DecisionReviewState::Suggested => GraphReviewState::Suggested,
        DecisionReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        DecisionReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

fn decision_graph_evidence(
    decision: &Decision,
    evidence: &[NewDecisionEvidence],
) -> NewGraphEvidence {
    let first_evidence = evidence.first();
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Decision,
        decision.decision_id.clone(),
    )
    .metadata(json!({
        "domain": "decision",
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

fn validate_decision_with_evidence(
    decision: &NewDecision,
    evidence: &[NewDecisionEvidence],
    impacted_entities: &[NewDecisionImpactedEntity],
) -> Result<(), DecisionStoreError> {
    decision.validate()?;
    if evidence.is_empty() {
        return Err(DecisionStoreError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }
    for item in impacted_entities {
        item.validate()?;
    }

    Ok(())
}

async fn preserve_existing_review_state(
    pool: &PgPool,
    decision: &mut NewDecision,
) -> Result<(), DecisionStoreError> {
    let existing_review_state: Option<String> =
        sqlx::query_scalar("SELECT review_state FROM decisions WHERE decision_id = $1")
            .bind(decision_id(decision))
            .fetch_optional(pool)
            .await?;
    let Some(existing_review_state) = existing_review_state else {
        return Ok(());
    };
    let existing_review_state = DecisionReviewState::parse(existing_review_state)?;
    if existing_review_state != DecisionReviewState::Suggested {
        decision.review_state = existing_review_state;
    }

    Ok(())
}

fn validate_refresh_limit(limit: i64) -> Result<i64, DecisionStoreError> {
    if !(MIN_REFRESH_LIMIT..=MAX_REFRESH_LIMIT).contains(&limit) {
        return Err(DecisionStoreError::InvalidLimit);
    }

    Ok(limit)
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), DecisionStoreError> {
    if value.trim().is_empty() {
        return Err(DecisionStoreError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_score(field_name: &'static str, value: f64) -> Result<(), DecisionStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(DecisionStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

fn validate_json_object(field_name: &'static str, value: &Value) -> Result<(), DecisionStoreError> {
    if !value.is_object() {
        return Err(DecisionStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}

fn validate_json_array(field_name: &'static str, value: &Value) -> Result<(), DecisionStoreError> {
    if !value.is_array() {
        return Err(DecisionStoreError::InvalidJsonArray(field_name));
    }

    Ok(())
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn row_to_decision(row: PgRow) -> Result<Decision, DecisionStoreError> {
    let decided_by_entity_kind = row
        .try_get::<Option<String>, _>("decided_by_entity_kind")?
        .map(parse_entity_kind)
        .transpose()?;

    Ok(Decision {
        decision_id: row.try_get("decision_id")?,
        title: row.try_get("title")?,
        status: parse_status(row.try_get("status")?)?,
        rationale: row.try_get("rationale")?,
        alternatives: row.try_get("alternatives")?,
        decided_by_entity_kind,
        decided_by_entity_id: row.try_get("decided_by_entity_id")?,
        decided_at: row.try_get("decided_at")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<DecisionEntityKind, DecisionStoreError> {
    DecisionEntityKind::parse(value)
}

fn parse_status(value: String) -> Result<DecisionStatus, DecisionStoreError> {
    match value.as_str() {
        "active" => Ok(DecisionStatus::Active),
        "superseded" => Ok(DecisionStatus::Superseded),
        "reversed" => Ok(DecisionStatus::Reversed),
        "deprecated" => Ok(DecisionStatus::Deprecated),
        _ => Err(DecisionStoreError::UnknownStatus(value)),
    }
}

fn parse_review_state(value: String) -> Result<DecisionReviewState, DecisionStoreError> {
    DecisionReviewState::parse(value)
}
