use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::ReviewInboxError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewItemKind {
    NewPersona,
    NewOrganization,
    IdentityCandidate,
    ProjectLinkCandidate,
    ContradictionCandidate,
    PotentialTask,
    PotentialObligation,
    PotentialDecision,
    PotentialRelationship,
    PotentialProject,
    KnowledgeCandidate,
}

impl ReviewItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NewPersona => "new_persona",
            Self::NewOrganization => "new_organization",
            Self::IdentityCandidate => "identity_candidate",
            Self::ProjectLinkCandidate => "project_link_candidate",
            Self::ContradictionCandidate => "contradiction_candidate",
            Self::PotentialTask => "potential_task",
            Self::PotentialObligation => "potential_obligation",
            Self::PotentialDecision => "potential_decision",
            Self::PotentialRelationship => "potential_relationship",
            Self::PotentialProject => "potential_project",
            Self::KnowledgeCandidate => "knowledge_candidate",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReviewInboxError> {
        match value.as_ref() {
            "new_persona" | "new_person" => Ok(Self::NewPersona),
            "new_organization" => Ok(Self::NewOrganization),
            "identity_candidate" => Ok(Self::IdentityCandidate),
            "project_link_candidate" => Ok(Self::ProjectLinkCandidate),
            "contradiction_candidate" => Ok(Self::ContradictionCandidate),
            "potential_task" => Ok(Self::PotentialTask),
            "potential_obligation" => Ok(Self::PotentialObligation),
            "potential_decision" => Ok(Self::PotentialDecision),
            "potential_relationship" => Ok(Self::PotentialRelationship),
            "potential_project" => Ok(Self::PotentialProject),
            "knowledge_candidate" => Ok(Self::KnowledgeCandidate),
            unknown => Err(ReviewInboxError::UnknownItemKind(unknown.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewItemStatus {
    New,
    InReview,
    Approved,
    Promoted,
    Dismissed,
    Archived,
}

impl ReviewItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::InReview => "in_review",
            Self::Approved => "approved",
            Self::Promoted => "promoted",
            Self::Dismissed => "dismissed",
            Self::Archived => "archived",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReviewInboxError> {
        match value.as_ref() {
            "new" => Ok(Self::New),
            "in_review" => Ok(Self::InReview),
            "approved" => Ok(Self::Approved),
            "promoted" => Ok(Self::Promoted),
            "dismissed" => Ok(Self::Dismissed),
            "archived" => Ok(Self::Archived),
            unknown => Err(ReviewInboxError::UnknownStatus(unknown.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReviewItem {
    pub review_item_id: String,
    pub item_kind: ReviewItemKind,
    pub title: String,
    pub summary: String,
    pub status: ReviewItemStatus,
    pub target_domain: Option<String>,
    pub target_entity_kind: Option<String>,
    pub target_entity_id: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewReviewItem {
    pub item_kind: ReviewItemKind,
    pub title: String,
    pub summary: String,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewReviewItem {
    pub fn new(
        item_kind: ReviewItemKind,
        title: impl Into<String>,
        summary: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            item_kind,
            title: title.into(),
            summary: summary.into(),
            confidence,
            metadata: json!({}),
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> Result<(), ReviewInboxError> {
        validate_non_empty("title", &self.title)?;
        validate_non_empty("summary", &self.summary)?;
        validate_score("confidence", self.confidence)?;
        validate_json_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReviewItemEvidence {
    pub review_item_id: String,
    pub observation_id: String,
    pub evidence_role: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewReviewItemEvidence {
    pub observation_id: String,
    pub evidence_role: String,
    pub metadata: Value,
}

impl NewReviewItemEvidence {
    pub fn new(observation_id: impl Into<String>) -> Self {
        Self {
            observation_id: observation_id.into(),
            evidence_role: "primary".to_owned(),
            metadata: json!({}),
        }
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.evidence_role = role.into();
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> Result<(), ReviewInboxError> {
        validate_non_empty("observation_id", &self.observation_id)?;
        validate_non_empty("evidence_role", &self.evidence_role)?;
        validate_json_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReviewPromotionTarget {
    pub target_domain: String,
    pub target_entity_kind: String,
    pub target_entity_id: String,
}

impl ReviewPromotionTarget {
    pub fn new(
        target_domain: impl Into<String>,
        target_entity_kind: impl Into<String>,
        target_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            target_domain: target_domain.into(),
            target_entity_kind: target_entity_kind.into(),
            target_entity_id: target_entity_id.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ReviewInboxError> {
        validate_non_empty("target_domain", &self.target_domain)?;
        validate_non_empty("target_entity_kind", &self.target_entity_kind)?;
        validate_non_empty("target_entity_id", &self.target_entity_id)?;
        Ok(())
    }
}

pub(super) fn validate_review_item_with_evidence(
    item: &NewReviewItem,
    evidence: &[NewReviewItemEvidence],
) -> Result<(), ReviewInboxError> {
    item.validate()?;
    if evidence.is_empty() {
        return Err(ReviewInboxError::MissingEvidence);
    }
    for item in evidence {
        item.validate()?;
    }
    Ok(())
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ReviewInboxError> {
    if value.trim().is_empty() {
        return Err(ReviewInboxError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ReviewInboxError> {
    if !value.is_object() {
        return Err(ReviewInboxError::InvalidJsonObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_score(field_name: &'static str, value: f64) -> Result<(), ReviewInboxError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(ReviewInboxError::InvalidScore(field_name, value));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ReviewItemKind;

    #[test]
    fn review_item_kind_writes_persona_native_value() {
        assert_eq!(ReviewItemKind::NewPersona.as_str(), "new_persona");
    }

    #[test]
    fn review_item_kind_reads_legacy_person_value() {
        assert_eq!(
            ReviewItemKind::parse("new_person").expect("legacy new_person should parse"),
            ReviewItemKind::NewPersona
        );
    }
}
