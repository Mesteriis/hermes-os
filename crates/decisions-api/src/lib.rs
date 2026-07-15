use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecisionRead {
    pub decision_id: String,
    pub title: String,
    pub status: String,
    pub rationale: String,
    pub alternatives: Value,
    pub decided_by_entity_kind: Option<String>,
    pub decided_by_entity_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub review_state: String,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct DecisionListQuery {
    pub entity_kind: Option<String>,
    pub entity_id: Option<String>,
    pub review_state: Option<String>,
    pub limit: Option<i64>,
}

pub type DecisionListFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<DecisionRead>, DecisionQueryError>> + Send + 'a>>;

pub trait DecisionReadPort: Send + Sync {
    fn list<'a>(&'a self, query: DecisionListQuery) -> DecisionListFuture<'a>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionUpsert {
    pub decision_id: String,
    pub title: String,
    pub status: String,
    pub rationale: String,
    pub alternatives: Value,
    pub decided_by_entity_kind: Option<String>,
    pub decided_by_entity_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub review_state: String,
    pub confidence: f64,
    pub metadata: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionEvidence {
    pub source_kind: String,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionImpactedEntity {
    pub entity_kind: String,
    pub entity_id: String,
    pub impact_type: String,
    pub metadata: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum DecisionWriteError {
    #[error("invalid decision write: {0}")]
    InvalidWrite(&'static str),
    #[error("decision write failed: {0}")]
    Failed(String),
}

pub type DecisionWriteFuture<'a> =
    Pin<Box<dyn Future<Output = Result<DecisionRead, DecisionWriteError>> + Send + 'a>>;
pub trait DecisionWritePort: Send + Sync {
    fn upsert<'a>(
        &'a self,
        decision: &'a DecisionUpsert,
        evidence: &'a [DecisionEvidence],
        impacted_entities: &'a [DecisionImpactedEntity],
    ) -> DecisionWriteFuture<'a>;
}

#[derive(Debug, thiserror::Error)]
#[error("decision query failed: {0}")]
pub struct DecisionQueryError(pub String);
