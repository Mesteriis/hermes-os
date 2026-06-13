use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::DecisionStoreError;
use super::validation::{
    validate_json_array, validate_json_object, validate_non_empty, validate_score,
};

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

    pub(super) fn validate(&self) -> Result<(), DecisionStoreError> {
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

    pub(super) fn validate(&self) -> Result<(), DecisionStoreError> {
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

    pub(super) fn validate(&self) -> Result<(), DecisionStoreError> {
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
