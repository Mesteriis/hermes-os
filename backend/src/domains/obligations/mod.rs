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

pub mod api;

#[derive(Clone)]
pub struct ObligationStore {
    pool: PgPool,
}

impl ObligationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_with_evidence(
        &self,
        obligation: &NewObligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<Obligation, ObligationStoreError> {
        validate_obligation_with_evidence(obligation, evidence)?;

        let mut transaction = self.pool.begin().await?;
        let stored =
            Self::upsert_with_evidence_in_transaction(&mut transaction, obligation, evidence)
                .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(crate) async fn upsert_with_evidence_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        obligation: &NewObligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<Obligation, ObligationStoreError> {
        let obligation_id = obligation_id(obligation);
        let row = sqlx::query(
            r#"
            INSERT INTO obligations (
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
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
                $10,
                $11,
                CAST($12 AS NUMERIC(5,4)),
                $13
            )
            ON CONFLICT (obligation_id)
            DO UPDATE SET
                status = EXCLUDED.status,
                review_state = EXCLUDED.review_state,
                due_at = EXCLUDED.due_at,
                condition = EXCLUDED.condition,
                risk_state = EXCLUDED.risk_state,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&obligation_id)
        .bind(obligation.obligated_entity_kind.as_str())
        .bind(&obligation.obligated_entity_id)
        .bind(obligation.beneficiary_entity_kind.map(|kind| kind.as_str()))
        .bind(&obligation.beneficiary_entity_id)
        .bind(&obligation.statement)
        .bind(obligation.status.as_str())
        .bind(obligation.review_state.as_str())
        .bind(obligation.due_at)
        .bind(&obligation.condition)
        .bind(obligation.risk_state.as_str())
        .bind(obligation.confidence)
        .bind(&obligation.metadata)
        .fetch_one(&mut **transaction)
        .await?;

        let stored = row_to_obligation(row)?;

        for item in evidence {
            let evidence_id = evidence_id(&obligation_id, item.source_kind, &item.source_id);
            sqlx::query(
                r#"
                INSERT INTO obligation_evidence (
                    evidence_id,
                    obligation_id,
                    source_kind,
                    source_id,
                    quote,
                    confidence,
                    metadata
                )
                VALUES ($1, $2, $3, $4, $5, CAST($6 AS NUMERIC(5,4)), $7)
                ON CONFLICT (obligation_id, source_kind, source_id)
                DO UPDATE SET
                    quote = EXCLUDED.quote,
                    confidence = EXCLUDED.confidence,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(evidence_id)
            .bind(&obligation_id)
            .bind(item.source_kind.as_str())
            .bind(&item.source_id)
            .bind(&item.quote)
            .bind(item.confidence)
            .bind(&item.metadata)
            .execute(&mut **transaction)
            .await?;
        }

        Self::project_obligation_graph_in_transaction(transaction, &stored, evidence).await?;

        Ok(stored)
    }

    async fn project_obligation_graph_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        obligation: &Obligation,
        evidence: &[NewObligationEvidence],
    ) -> Result<(), ObligationStoreError> {
        let obligation_node = NewGraphNode::new(
            GraphNodeKind::Obligation,
            obligation.obligation_id.clone(),
            obligation.statement.clone(),
        )
        .properties(json!({
            "domain": "obligation",
            "obligation_id": obligation.obligation_id,
            "status": obligation.status.as_str(),
            "review_state": obligation.review_state.as_str(),
            "risk_state": obligation.risk_state.as_str(),
        }));
        let stored_obligation_node =
            GraphStore::upsert_node_in_transaction(transaction, &obligation_node).await?;

        Self::project_obligation_entity_edge_in_transaction(
            transaction,
            obligation,
            evidence,
            &stored_obligation_node.node_id,
            obligation.obligated_entity_kind,
            &obligation.obligated_entity_id,
            "obligated_entity",
        )
        .await?;

        if let (Some(beneficiary_entity_kind), Some(beneficiary_entity_id)) = (
            obligation.beneficiary_entity_kind,
            obligation.beneficiary_entity_id.as_deref(),
        ) {
            Self::project_obligation_entity_edge_in_transaction(
                transaction,
                obligation,
                evidence,
                &stored_obligation_node.node_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                "beneficiary_entity",
            )
            .await?;
        }

        Ok(())
    }

    async fn project_obligation_entity_edge_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        obligation: &Obligation,
        evidence: &[NewObligationEvidence],
        obligation_node_id: &str,
        entity_kind: ObligationEntityKind,
        entity_id: &str,
        link_role: &str,
    ) -> Result<(), ObligationStoreError> {
        let Some(target_node_kind) = obligation_entity_to_graph_node_kind(entity_kind) else {
            return Ok(());
        };
        let target_node =
            NewGraphNode::new(target_node_kind, entity_id, entity_id).properties(json!({
                "domain": entity_kind.as_str(),
                "entity_id": entity_id,
            }));
        let stored_target_node =
            GraphStore::upsert_node_in_transaction(transaction, &target_node).await?;

        let graph_edge = NewGraphEdge::new(
            obligation_node_id.to_owned(),
            stored_target_node.node_id,
            GraphRelationshipType::EntityRelationship,
            obligation.confidence,
            obligation_review_state_to_graph_review_state(obligation.review_state),
        )
        .properties(json!({
            "domain": "obligation",
            "obligation_id": obligation.obligation_id,
            "link_role": link_role,
            "status": obligation.status.as_str(),
            "risk_state": obligation.risk_state.as_str(),
        }));
        let graph_evidence = obligation_graph_evidence(obligation, evidence);

        GraphStore::upsert_edge_with_evidence_in_transaction(
            transaction,
            &graph_edge,
            &[graph_evidence],
        )
        .await?;

        Ok(())
    }

    pub async fn list_for_entity(
        &self,
        entity_kind: ObligationEntityKind,
        entity_id: &str,
        limit: i64,
    ) -> Result<Vec<Obligation>, ObligationStoreError> {
        validate_non_empty("entity_id", entity_id)?;
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
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM obligations
            WHERE (obligated_entity_kind = $1 AND obligated_entity_id = $2)
               OR (beneficiary_entity_kind = $1 AND beneficiary_entity_id = $2)
            ORDER BY updated_at DESC, obligation_id ASC
            LIMIT $3
            "#,
        )
        .bind(entity_kind.as_str())
        .bind(entity_id)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_obligation).collect()
    }

    pub async fn list_by_review_state(
        &self,
        review_state: ObligationReviewState,
        limit: i64,
    ) -> Result<Vec<Obligation>, ObligationStoreError> {
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
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            FROM obligations
            WHERE review_state = $1
            ORDER BY updated_at DESC, obligation_id ASC
            LIMIT $2
            "#,
        )
        .bind(review_state.as_str())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_obligation).collect()
    }

    pub async fn set_review_state(
        &self,
        obligation_id: &str,
        review_state: ObligationReviewState,
    ) -> Result<Obligation, ObligationStoreError> {
        validate_non_empty("obligation_id", obligation_id)?;
        let row = sqlx::query(
            r#"
            UPDATE obligations
            SET
                review_state = $1,
                updated_at = now()
            WHERE obligation_id = $2
            RETURNING
                obligation_id,
                obligated_entity_kind,
                obligated_entity_id,
                beneficiary_entity_kind,
                beneficiary_entity_id,
                statement,
                status,
                review_state,
                due_at,
                condition,
                risk_state,
                confidence::float8 AS confidence,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(review_state.as_str())
        .bind(obligation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ObligationStoreError::ObligationNotFound)?;

        row_to_obligation(row)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationEntityKind {
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

impl ObligationEntityKind {
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

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObligationStoreError> {
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
            _ => Err(ObligationStoreError::UnknownEntityKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationEvidenceSourceKind {
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

impl ObligationEvidenceSourceKind {
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
pub enum ObligationStatus {
    Open,
    Fulfilled,
    Waived,
    Disputed,
    Canceled,
}

impl ObligationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Fulfilled => "fulfilled",
            Self::Waived => "waived",
            Self::Disputed => "disputed",
            Self::Canceled => "canceled",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ObligationReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObligationStoreError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ObligationStoreError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationRiskState {
    None,
    Watch,
    AtRisk,
    Breached,
}

impl ObligationRiskState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Watch => "watch",
            Self::AtRisk => "at_risk",
            Self::Breached => "breached",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewObligation {
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub status: ObligationStatus,
    pub review_state: ObligationReviewState,
    pub due_at: Option<DateTime<Utc>>,
    pub condition: Option<String>,
    pub risk_state: ObligationRiskState,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObligation {
    pub fn new(
        obligated_entity_kind: ObligationEntityKind,
        obligated_entity_id: impl Into<String>,
        statement: impl Into<String>,
        confidence: f64,
        review_state: ObligationReviewState,
    ) -> Self {
        Self {
            obligated_entity_kind,
            obligated_entity_id: obligated_entity_id.into(),
            beneficiary_entity_kind: None,
            beneficiary_entity_id: None,
            statement: statement.into(),
            status: ObligationStatus::Open,
            review_state,
            due_at: None,
            condition: None,
            risk_state: ObligationRiskState::None,
            confidence,
            metadata: json!({}),
        }
    }

    pub fn beneficiary(
        mut self,
        beneficiary_entity_kind: ObligationEntityKind,
        beneficiary_entity_id: impl Into<String>,
    ) -> Self {
        self.beneficiary_entity_kind = Some(beneficiary_entity_kind);
        self.beneficiary_entity_id = Some(beneficiary_entity_id.into());
        self
    }

    pub fn status(mut self, status: ObligationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn due_at(mut self, due_at: DateTime<Utc>) -> Self {
        self.due_at = Some(due_at);
        self
    }

    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    pub fn risk_state(mut self, risk_state: ObligationRiskState) -> Self {
        self.risk_state = risk_state;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    fn validate(&self) -> Result<(), ObligationStoreError> {
        validate_non_empty("obligated_entity_id", &self.obligated_entity_id)?;
        validate_non_empty("statement", &self.statement)?;
        validate_score("confidence", self.confidence)?;
        validate_json_object("obligation metadata", &self.metadata)?;

        match (
            self.beneficiary_entity_kind,
            self.beneficiary_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(beneficiary_entity_id)) => {
                validate_non_empty("beneficiary_entity_id", beneficiary_entity_id)?;
            }
            _ => return Err(ObligationStoreError::PartialBeneficiary),
        }

        if let Some(condition) = &self.condition {
            validate_non_empty("condition", condition)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewObligationEvidence {
    pub source_kind: ObligationEvidenceSourceKind,
    pub source_id: String,
    pub quote: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObligationEvidence {
    pub fn new(source_kind: ObligationEvidenceSourceKind, source_id: impl Into<String>) -> Self {
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

    fn validate(&self) -> Result<(), ObligationStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_score("evidence confidence", self.confidence)?;
        validate_json_object("evidence metadata", &self.metadata)?;
        if let Some(quote) = &self.quote {
            validate_non_empty("quote", quote)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Obligation {
    pub obligation_id: String,
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub status: ObligationStatus,
    pub review_state: ObligationReviewState,
    pub due_at: Option<DateTime<Utc>>,
    pub condition: Option<String>,
    pub risk_state: ObligationRiskState,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum ObligationStoreError {
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

    #[error("obligation evidence is required")]
    MissingEvidence,

    #[error("obligation was not found")]
    ObligationNotFound,

    #[error("beneficiary entity kind and id must be provided together")]
    PartialBeneficiary,

    #[error("unknown obligation entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown obligation evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown obligation status stored in database: {0}")]
    UnknownStatus(String),

    #[error("unknown obligation review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("unknown obligation risk state stored in database: {0}")]
    UnknownRiskState(String),
}

pub fn obligation_id(obligation: &NewObligation) -> String {
    let beneficiary_kind = obligation
        .beneficiary_entity_kind
        .map(ObligationEntityKind::as_str)
        .unwrap_or("");
    let beneficiary_id = obligation.beneficiary_entity_id.as_deref().unwrap_or("");
    let statement = normalize_statement(&obligation.statement);

    format!(
        "obligation:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        obligation.obligated_entity_kind.as_str().len(),
        obligation.obligated_entity_kind.as_str(),
        obligation.obligated_entity_id.len(),
        obligation.obligated_entity_id,
        beneficiary_kind.len(),
        beneficiary_kind,
        beneficiary_id.len(),
        beneficiary_id,
        statement.len(),
        statement
    )
}

pub fn evidence_id(
    obligation_id: &str,
    source_kind: ObligationEvidenceSourceKind,
    source_id: &str,
) -> String {
    format!(
        "obligation:evidence:v1:{}:{}:{}:{}:{}:{}",
        obligation_id.len(),
        obligation_id,
        source_kind.as_str().len(),
        source_kind.as_str(),
        source_id.len(),
        source_id
    )
}

fn obligation_entity_to_graph_node_kind(
    entity_kind: ObligationEntityKind,
) -> Option<GraphNodeKind> {
    match entity_kind {
        ObligationEntityKind::Persona => Some(GraphNodeKind::Person),
        ObligationEntityKind::Project => Some(GraphNodeKind::Project),
        ObligationEntityKind::Communication => Some(GraphNodeKind::Message),
        ObligationEntityKind::Document => Some(GraphNodeKind::Document),
        ObligationEntityKind::Decision => Some(GraphNodeKind::Decision),
        ObligationEntityKind::Obligation => Some(GraphNodeKind::Obligation),
        ObligationEntityKind::Organization
        | ObligationEntityKind::Task
        | ObligationEntityKind::Event
        | ObligationEntityKind::Knowledge => None,
    }
}

fn obligation_review_state_to_graph_review_state(
    review_state: ObligationReviewState,
) -> GraphReviewState {
    match review_state {
        ObligationReviewState::Suggested => GraphReviewState::Suggested,
        ObligationReviewState::UserConfirmed => GraphReviewState::UserConfirmed,
        ObligationReviewState::UserRejected => GraphReviewState::UserRejected,
    }
}

fn obligation_graph_evidence(
    obligation: &Obligation,
    evidence: &[NewObligationEvidence],
) -> NewGraphEvidence {
    let first_evidence = evidence.first();
    let mut graph_evidence = NewGraphEvidence::new(
        GraphEvidenceSourceKind::Obligation,
        obligation.obligation_id.clone(),
    )
    .metadata(json!({
        "domain": "obligation",
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

fn validate_obligation_with_evidence(
    obligation: &NewObligation,
    evidence: &[NewObligationEvidence],
) -> Result<(), ObligationStoreError> {
    obligation.validate()?;
    if evidence.is_empty() {
        return Err(ObligationStoreError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }

    Ok(())
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), ObligationStoreError> {
    if value.trim().is_empty() {
        return Err(ObligationStoreError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_score(field_name: &'static str, value: f64) -> Result<(), ObligationStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(ObligationStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}

fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ObligationStoreError> {
    if !value.is_object() {
        return Err(ObligationStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}

fn normalize_statement(statement: &str) -> String {
    statement
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn row_to_obligation(row: PgRow) -> Result<Obligation, ObligationStoreError> {
    let beneficiary_entity_kind = row
        .try_get::<Option<String>, _>("beneficiary_entity_kind")?
        .map(parse_entity_kind)
        .transpose()?;

    Ok(Obligation {
        obligation_id: row.try_get("obligation_id")?,
        obligated_entity_kind: parse_entity_kind(row.try_get("obligated_entity_kind")?)?,
        obligated_entity_id: row.try_get("obligated_entity_id")?,
        beneficiary_entity_kind,
        beneficiary_entity_id: row.try_get("beneficiary_entity_id")?,
        statement: row.try_get("statement")?,
        status: parse_status(row.try_get("status")?)?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        due_at: row.try_get("due_at")?,
        condition: row.try_get("condition")?,
        risk_state: parse_risk_state(row.try_get("risk_state")?)?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_entity_kind(value: String) -> Result<ObligationEntityKind, ObligationStoreError> {
    ObligationEntityKind::parse(value)
}

fn parse_status(value: String) -> Result<ObligationStatus, ObligationStoreError> {
    match value.as_str() {
        "open" => Ok(ObligationStatus::Open),
        "fulfilled" => Ok(ObligationStatus::Fulfilled),
        "waived" => Ok(ObligationStatus::Waived),
        "disputed" => Ok(ObligationStatus::Disputed),
        "canceled" => Ok(ObligationStatus::Canceled),
        _ => Err(ObligationStoreError::UnknownStatus(value)),
    }
}

fn parse_review_state(value: String) -> Result<ObligationReviewState, ObligationStoreError> {
    ObligationReviewState::parse(value)
}

fn parse_risk_state(value: String) -> Result<ObligationRiskState, ObligationStoreError> {
    match value.as_str() {
        "none" => Ok(ObligationRiskState::None),
        "watch" => Ok(ObligationRiskState::Watch),
        "at_risk" => Ok(ObligationRiskState::AtRisk),
        "breached" => Ok(ObligationRiskState::Breached),
        _ => Err(ObligationStoreError::UnknownRiskState(value)),
    }
}
