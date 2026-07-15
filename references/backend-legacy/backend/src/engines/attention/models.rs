use serde::{Deserialize, Serialize};

use super::errors::AttentionEngineError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionImportance {
    Low,
    Medium,
    High,
    Critical,
}

impl AttentionImportance {
    pub(crate) fn rank(self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Critical => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttentionEvidenceRef {
    pub observation_id: String,
    pub role: String,
}

impl AttentionEvidenceRef {
    pub fn new(observation_id: impl Into<String>) -> Self {
        Self {
            observation_id: observation_id.into(),
            role: "primary".to_owned(),
        }
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }

    pub(crate) fn validate(&self) -> Result<(), AttentionEngineError> {
        validate_non_empty("observation_id", &self.observation_id)?;
        validate_non_empty("evidence_role", &self.role)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttentionRelatedEntity {
    pub entity_kind: String,
    pub entity_id: String,
    pub label: Option<String>,
}

impl AttentionRelatedEntity {
    pub fn new(entity_kind: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self {
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<(), AttentionEngineError> {
        validate_non_empty("related_entity_kind", &self.entity_kind)?;
        validate_non_empty("related_entity_id", &self.entity_id)?;
        if let Some(label) = &self.label {
            validate_non_empty("related_entity_label", label)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttentionSuggestedAction {
    pub action_kind: String,
    pub label: String,
    pub target_domain: Option<String>,
    pub target_entity_kind: Option<String>,
}

impl AttentionSuggestedAction {
    pub fn new(action_kind: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            action_kind: action_kind.into(),
            label: label.into(),
            target_domain: None,
            target_entity_kind: None,
        }
    }

    pub fn target(
        mut self,
        target_domain: impl Into<String>,
        target_entity_kind: impl Into<String>,
    ) -> Self {
        self.target_domain = Some(target_domain.into());
        self.target_entity_kind = Some(target_entity_kind.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<(), AttentionEngineError> {
        validate_non_empty("action_kind", &self.action_kind)?;
        validate_non_empty("action_label", &self.label)?;
        if let Some(target_domain) = &self.target_domain {
            validate_non_empty("target_domain", target_domain)?;
        }
        if let Some(target_entity_kind) = &self.target_entity_kind {
            validate_non_empty("target_entity_kind", target_entity_kind)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttentionCandidate {
    pub review_item_id: String,
    pub candidate_kind: String,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub confidence: f64,
    pub evidence: Vec<AttentionEvidenceRef>,
    pub related_entities: Vec<AttentionRelatedEntity>,
    pub trace_id: String,
    pub group_key: Option<String>,
    pub source_summary: String,
    pub suggested_actions: Vec<AttentionSuggestedAction>,
}

impl AttentionCandidate {
    pub fn new(
        review_item_id: impl Into<String>,
        candidate_kind: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        trace_id: impl Into<String>,
    ) -> Self {
        Self {
            review_item_id: review_item_id.into(),
            candidate_kind: candidate_kind.into(),
            title: title.into(),
            summary: summary.into(),
            status: "new".to_owned(),
            confidence: 0.0,
            evidence: Vec::new(),
            related_entities: Vec::new(),
            trace_id: trace_id.into(),
            group_key: None,
            source_summary: String::new(),
            suggested_actions: Vec::new(),
        }
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn evidence(mut self, evidence: Vec<AttentionEvidenceRef>) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn related_entities(mut self, related_entities: Vec<AttentionRelatedEntity>) -> Self {
        self.related_entities = related_entities;
        self
    }

    pub fn group_key(mut self, group_key: impl Into<String>) -> Self {
        self.group_key = Some(group_key.into());
        self
    }

    pub fn source_summary(mut self, source_summary: impl Into<String>) -> Self {
        self.source_summary = source_summary.into();
        self
    }

    pub fn suggested_actions(mut self, suggested_actions: Vec<AttentionSuggestedAction>) -> Self {
        self.suggested_actions = suggested_actions;
        self
    }

    pub(crate) fn normalized_group_key(&self) -> Result<String, AttentionEngineError> {
        match &self.group_key {
            Some(group_key) => {
                validate_non_empty("group_key", group_key)?;
                Ok(group_key.trim().to_owned())
            }
            None => Ok(self.review_item_id.trim().to_owned()),
        }
    }

    pub(crate) fn review_status(&self) -> Result<AttentionReviewStatus, AttentionEngineError> {
        AttentionReviewStatus::parse(&self.status)
    }

    pub(crate) fn validate(&self) -> Result<(), AttentionEngineError> {
        validate_non_empty("review_item_id", &self.review_item_id)?;
        validate_non_empty("candidate_kind", &self.candidate_kind)?;
        validate_non_empty("title", &self.title)?;
        validate_non_empty("summary", &self.summary)?;
        validate_non_empty("status", &self.status)?;
        validate_non_empty("trace_id", &self.trace_id)?;
        validate_non_empty("source_summary", &self.source_summary)?;
        validate_confidence(self.confidence)?;
        self.review_status()?;
        self.normalized_group_key()?;

        if self.evidence.is_empty() {
            return Err(AttentionEngineError::MissingEvidence(
                self.review_item_id.clone(),
            ));
        }
        for evidence in &self.evidence {
            evidence.validate()?;
        }

        for entity in &self.related_entities {
            entity.validate()?;
        }

        if self.suggested_actions.is_empty() {
            return Err(AttentionEngineError::MissingSuggestedActions(
                self.review_item_id.clone(),
            ));
        }
        for action in &self.suggested_actions {
            action.validate()?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttentionReviewStatus {
    New,
    InReview,
    Approved,
    Promoted,
    Dismissed,
    Archived,
}

impl AttentionReviewStatus {
    fn parse(value: &str) -> Result<Self, AttentionEngineError> {
        match value.trim() {
            "new" => Ok(Self::New),
            "in_review" => Ok(Self::InReview),
            "approved" => Ok(Self::Approved),
            "promoted" => Ok(Self::Promoted),
            "dismissed" => Ok(Self::Dismissed),
            "archived" => Ok(Self::Archived),
            unknown => Err(AttentionEngineError::UnknownReviewStatus(
                unknown.to_owned(),
            )),
        }
    }

    pub(crate) fn is_active(self) -> bool {
        matches!(self, Self::New | Self::InReview | Self::Approved)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttentionConfidenceExplanation {
    pub score: f64,
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttentionExplainability {
    pub why_this_matters: String,
    pub evidence: Vec<AttentionEvidenceRef>,
    pub confidence: AttentionConfidenceExplanation,
    pub related_objects: Vec<AttentionRelatedEntity>,
    pub suggested_actions: Vec<AttentionSuggestedAction>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttentionCard {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub importance: AttentionImportance,
    pub confidence: f64,
    pub evidence_count: usize,
    pub related_entities: Vec<AttentionRelatedEntity>,
    pub trace_id: String,
    pub review_item_ids: Vec<String>,
    pub suggested_actions: Vec<AttentionSuggestedAction>,
    pub source_summary: String,
    pub explainability: AttentionExplainability,
}

pub(crate) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), AttentionEngineError> {
    if value.trim().is_empty() {
        return Err(AttentionEngineError::EmptyField(field));
    }
    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), AttentionEngineError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(AttentionEngineError::InvalidConfidence(confidence));
    }
    Ok(())
}
