use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObligationRead {
    pub obligation_id: String,
    pub obligated_entity_kind: String,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<String>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub status: String,
    pub review_state: String,
    pub due_at: Option<DateTime<Utc>>,
    pub condition: Option<String>,
    pub risk_state: String,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ObligationListQuery {
    pub entity_kind: Option<String>,
    pub entity_id: Option<String>,
    pub review_state: Option<String>,
    pub limit: Option<i64>,
}

pub type ObligationListFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<ObligationRead>, ObligationQueryError>> + Send + 'a>>;
pub trait ObligationReadPort: Send + Sync {
    fn list<'a>(&'a self, query: ObligationListQuery) -> ObligationListFuture<'a>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObligationUpsert {
    pub obligation_id: String,
    pub obligated_entity_kind: String,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<String>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub status: String,
    pub review_state: String,
    pub due_at: Option<DateTime<Utc>>,
    pub condition: Option<String>,
    pub risk_state: String,
    pub confidence: f64,
    pub metadata: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObligationEvidence {
    pub source_kind: String,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

pub type ObligationWriteFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ObligationRead, ObligationWriteError>> + Send + 'a>>;
pub trait ObligationWritePort: Send + Sync {
    fn upsert<'a>(
        &'a self,
        obligation: &'a ObligationUpsert,
        evidence: &'a [ObligationEvidence],
    ) -> ObligationWriteFuture<'a>;
}

#[derive(Debug, thiserror::Error)]
pub enum ObligationWriteError {
    #[error("invalid obligation write: {0}")]
    InvalidWrite(&'static str),
    #[error("obligation write failed: {0}")]
    Failed(String),
}

#[derive(Debug, thiserror::Error)]
#[error("obligation query failed: {0}")]
pub struct ObligationQueryError(pub String);
