use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationshipUpsert {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipEvidenceSourceKind {
    Observation,
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
}
impl RelationshipEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
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
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationshipEvidence {
    pub source_kind: RelationshipEvidenceSourceKind,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

pub type RelationshipWriteFuture<'a> =
    Pin<Box<dyn Future<Output = Result<RelationshipRead, RelationshipWriteError>> + Send + 'a>>;
pub trait RelationshipWritePort: Send + Sync {
    fn upsert<'a>(
        &'a self,
        relationship: &'a RelationshipUpsert,
        evidence: &'a [RelationshipEvidence],
    ) -> RelationshipWriteFuture<'a>;
}

pub type RelationshipListFuture<'a> = Pin<
    Box<dyn Future<Output = Result<Vec<RelationshipRead>, RelationshipQueryError>> + Send + 'a>,
>;

pub trait RelationshipQueryPort: Send + Sync {
    fn list<'a>(&'a self, query: RelationshipListQuery) -> RelationshipListFuture<'a>;
}

pub type RelationshipReviewFuture<'a> =
    Pin<Box<dyn Future<Output = Result<RelationshipRead, RelationshipReviewError>> + Send + 'a>>;

pub trait RelationshipReviewPort: Send + Sync {
    fn review<'a>(
        &'a self,
        relationship_id: &'a str,
        request: RelationshipReviewRequest,
    ) -> RelationshipReviewFuture<'a>;
}

#[derive(Debug, thiserror::Error)]
pub enum RelationshipQueryError {
    #[error("invalid relationship query: {0}")]
    InvalidQuery(&'static str),
    #[error("relationship query failed: {0}")]
    Failed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RelationshipReviewError {
    #[error("invalid relationship review: {0}")]
    InvalidReview(&'static str),
    #[error("relationship review failed: {0}")]
    Failed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RelationshipWriteError {
    #[error("invalid relationship write: {0}")]
    InvalidWrite(&'static str),
    #[error("relationship write failed: {0}")]
    Failed(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RelationshipRead {
    pub relationship_id: String,
    pub source_entity_kind: String,
    pub source_entity_id: String,
    pub target_entity_kind: String,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub trust_score: f64,
    pub strength_score: f64,
    pub confidence: f64,
    pub review_state: String,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct RelationshipListQuery {
    pub entity_kind: Option<String>,
    pub entity_id: Option<String>,
    pub review_state: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct RelationshipReviewRequest {
    pub review_state: String,
}

#[cfg(test)]
mod tests {
    use super::RelationshipRead;
    use super::{RelationshipEntityKind, RelationshipEvidenceSourceKind, RelationshipReviewState};
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn relationship_read_serializes_stable_external_shape() {
        let timestamp = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        let value = serde_json::to_value(RelationshipRead {
            relationship_id: "rel-1".to_owned(),
            source_entity_kind: "persona".to_owned(),
            source_entity_id: "persona-1".to_owned(),
            target_entity_kind: "project".to_owned(),
            target_entity_id: "project-1".to_owned(),
            relationship_type: "works_on".to_owned(),
            trust_score: 0.8,
            strength_score: 0.7,
            confidence: 0.9,
            review_state: "system_accepted".to_owned(),
            valid_from: Some(timestamp),
            valid_to: None,
            metadata: json!({"source": "fixture"}),
            created_at: timestamp,
            updated_at: timestamp,
        })
        .unwrap();

        assert_eq!(value["relationship_id"], "rel-1");
        assert_eq!(value["source_entity_kind"], "persona");
        assert_eq!(value["review_state"], "system_accepted");
        assert_eq!(value["metadata"]["source"], "fixture");
        assert!(value["valid_to"].is_null());
    }

    #[test]
    fn write_contract_preserves_persisted_relationship_tokens() {
        assert_eq!(
            RelationshipEntityKind::Communication.as_str(),
            "communication"
        );
        assert_eq!(
            RelationshipReviewState::SystemAccepted.as_str(),
            "system_accepted"
        );
        assert_eq!(
            RelationshipEvidenceSourceKind::Observation.as_str(),
            "observation"
        );
    }
}
