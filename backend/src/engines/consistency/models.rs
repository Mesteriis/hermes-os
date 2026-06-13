use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::ConsistencyError;
use super::validation::{
    validate_confidence, validate_json_array_or_object, validate_json_object, validate_non_empty,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSourceKind {
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Relationship,
    RawRecord,
}

impl ContradictionSourceKind {
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
            Self::RawRecord => "raw_record",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ContradictionSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ContradictionReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ConsistencyError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ConsistencyError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AcceptedClaim {
    pub subject_id: String,
    pub claim_type: String,
    pub value: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub confidence: f64,
}

impl AcceptedClaim {
    pub(super) fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("claim_type", &self.claim_type)?;
        validate_non_empty("value", &self.value)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewEvidenceClaim {
    pub subject_id: String,
    pub claim_type: String,
    pub value: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub confidence: f64,
}

impl NewEvidenceClaim {
    pub(super) fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("claim_type", &self.claim_type)?;
        validate_non_empty("value", &self.value)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceClaimExtractionInput {
    pub subject_id: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub text: String,
    pub confidence: f64,
}

impl EvidenceClaimExtractionInput {
    pub(super) fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("text", &self.text)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewContradictionObservation {
    pub old_source_kind: ContradictionSourceKind,
    pub old_source_id: String,
    pub new_source_kind: ContradictionSourceKind,
    pub new_source_id: String,
    pub affected_entities: Value,
    pub conflict_type: String,
    pub old_claim: String,
    pub new_claim: String,
    pub confidence: f64,
    pub severity: ContradictionSeverity,
    pub review_state: ContradictionReviewState,
    pub metadata: Value,
}

impl NewContradictionObservation {
    pub fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("old_source_id", &self.old_source_id)?;
        validate_non_empty("new_source_id", &self.new_source_id)?;
        validate_non_empty("conflict_type", &self.conflict_type)?;
        validate_non_empty("old_claim", &self.old_claim)?;
        validate_non_empty("new_claim", &self.new_claim)?;
        validate_confidence(self.confidence)?;
        validate_json_array_or_object("affected_entities", &self.affected_entities)?;
        validate_json_object("metadata", &self.metadata)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContradictionObservation {
    pub observation_id: String,
    pub old_source_kind: ContradictionSourceKind,
    pub old_source_id: String,
    pub new_source_kind: ContradictionSourceKind,
    pub new_source_id: String,
    pub affected_entities: Value,
    pub conflict_type: String,
    pub old_claim: String,
    pub new_claim: String,
    pub confidence: f64,
    pub severity: ContradictionSeverity,
    pub review_state: ContradictionReviewState,
    pub metadata: Value,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
