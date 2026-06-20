use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use crate::platform::graph::GraphNodeKind;

use super::errors::GraphStoreError;
use super::validation::{validate_json_object, validate_non_empty};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    PersonHasEmailAddress,
    PersonSentMessage,
    PersonReceivedMessage,
    EmailAddressSentMessage,
    EmailAddressReceivedMessage,
    ProjectHasMessage,
    ProjectHasDocument,
    ProjectInvolvesPerson,
    ProjectInvolvesEmailAddress,
    EntityRelationship,
}

impl RelationshipType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersonHasEmailAddress => "person_has_email_address",
            Self::PersonSentMessage => "person_sent_message",
            Self::PersonReceivedMessage => "person_received_message",
            Self::EmailAddressSentMessage => "email_address_sent_message",
            Self::EmailAddressReceivedMessage => "email_address_received_message",
            Self::ProjectHasMessage => "project_has_message",
            Self::ProjectHasDocument => "project_has_document",
            Self::ProjectInvolvesPerson => "project_involves_person",
            Self::ProjectInvolvesEmailAddress => "project_involves_email_address",
            Self::EntityRelationship => "entity_relationship",
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
    Person,
    Message,
    Document,
    Relationship,
    Decision,
    Obligation,
    Observation,
}

impl GraphEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Person => "contact",
            Self::Message => "message",
            Self::Document => "document",
            Self::Relationship => "relationship",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Observation => "observation",
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

    pub(super) fn validate(&self) -> Result<(), GraphStoreError> {
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

    pub(super) fn validate(&self) -> Result<(), GraphStoreError> {
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
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

impl NewGraphEvidence {
    pub fn new(source_kind: GraphEvidenceSourceKind, source_id: impl Into<String>) -> Self {
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
            source_kind: GraphEvidenceSourceKind::Observation,
            source_id: observation_id.clone(),
            observation_id: Some(observation_id),
            excerpt: None,
            metadata: json!({}),
        }
    }

    pub fn observation_id(mut self, observation_id: impl Into<String>) -> Self {
        self.observation_id = Some(observation_id.into());
        self
    }

    pub fn excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub(super) fn validate(&self) -> Result<(), GraphStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        if let Some(observation_id) = &self.observation_id {
            validate_non_empty("observation_id", observation_id)?;
        }
        if self.source_kind == GraphEvidenceSourceKind::Message && self.observation_id.is_none() {
            return Err(GraphStoreError::MissingObservationEvidence {
                source_kind: self.source_kind.as_str(),
            });
        }
        if self.source_kind == GraphEvidenceSourceKind::Observation
            && self.observation_id.as_deref() != Some(self.source_id.as_str())
        {
            return Err(GraphStoreError::ObservationSourceMismatch);
        }
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
    pub observation_id: Option<String>,
    pub excerpt: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphNeighborhood {
    pub selected_node: GraphNode,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub evidence: Vec<GraphEvidenceSummary>,
    pub edge_limit: i64,
    pub truncated: bool,
    pub evidence_limit: i64,
    pub evidence_truncated: bool,
}
