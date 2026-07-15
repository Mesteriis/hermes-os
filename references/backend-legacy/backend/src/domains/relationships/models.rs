use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::RelationshipStoreError;

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
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewRelationshipEvidence {
    pub source_kind: RelationshipEvidenceSourceKind,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

impl NewRelationshipEvidence {
    pub fn new(source_kind: RelationshipEvidenceSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            observation_id: None,
            excerpt: None,
            metadata: json!({}),
        }
    }

    pub fn observation(observation_id: impl Into<String>) -> Self {
        let observation_id = observation_id.into();
        Self {
            source_kind: RelationshipEvidenceSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id),
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
