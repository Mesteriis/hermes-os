use serde::{Deserialize, Serialize};
use serde_json::json;

use super::errors::DecisionEngineError;
use crate::domains::decisions::{
    DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, NewDecision,
    NewDecisionEvidence, NewDecisionImpactedEntity,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionExtractionInput {
    pub source_kind: DecisionEvidenceSourceKind,
    pub source_id: String,
    pub text: String,
    pub observation_id: Option<String>,
    pub impacted_entity_kind: DecisionEntityKind,
    pub impacted_entity_id: String,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
}

impl DecisionExtractionInput {
    pub fn communication(
        source_id: impl Into<String>,
        text: impl Into<String>,
        impacted_entity_kind: DecisionEntityKind,
        impacted_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_kind: DecisionEvidenceSourceKind::Communication,
            source_id: source_id.into(),
            text: text.into(),
            observation_id: None,
            impacted_entity_kind,
            impacted_entity_id: impacted_entity_id.into(),
            decided_by_entity_kind: None,
            decided_by_entity_id: None,
        }
    }

    pub fn document(
        source_id: impl Into<String>,
        text: impl Into<String>,
        impacted_entity_kind: DecisionEntityKind,
        impacted_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_kind: DecisionEvidenceSourceKind::Document,
            source_id: source_id.into(),
            text: text.into(),
            observation_id: None,
            impacted_entity_kind,
            impacted_entity_id: impacted_entity_id.into(),
            decided_by_entity_kind: None,
            decided_by_entity_id: None,
        }
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

    pub fn with_observation_id(mut self, observation_id: Option<String>) -> Self {
        self.observation_id = observation_id;
        self
    }

    pub fn validate(&self) -> Result<(), DecisionEngineError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("impacted_entity_id", &self.impacted_entity_id)?;
        if let Some(observation_id) = &self.observation_id {
            validate_non_empty("observation_id", observation_id)?;
        }
        match (
            self.decided_by_entity_kind,
            self.decided_by_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(decided_by_entity_id)) => {
                validate_non_empty("decided_by_entity_id", decided_by_entity_id)?;
            }
            _ => return Err(DecisionEngineError::PartialDecider),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecisionExtractionResult {
    pub decisions: Vec<DecisionCandidate>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionCandidateKind {
    ExplicitDecision,
    Approval,
    Confirmation,
}

impl DecisionCandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitDecision => "explicit_decision",
            Self::Approval => "approval",
            Self::Confirmation => "confirmation",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecisionCandidate {
    pub kind: DecisionCandidateKind,
    pub title: String,
    pub rationale: String,
    pub quote: String,
    pub confidence: f64,
    pub review_state: DecisionReviewState,
    pub evidence_source_kind: DecisionEvidenceSourceKind,
    pub evidence_source_id: String,
    pub evidence_observation_id: Option<String>,
    pub decided_by_entity_kind: Option<DecisionEntityKind>,
    pub decided_by_entity_id: Option<String>,
    pub impacted_entities: Vec<DecisionImpactedEntityCandidate>,
}

impl DecisionCandidate {
    pub fn to_decision_draft(
        &self,
    ) -> (
        NewDecision,
        NewDecisionEvidence,
        Vec<NewDecisionImpactedEntity>,
    ) {
        let mut decision = NewDecision::new(
            self.title.clone(),
            self.rationale.clone(),
            self.confidence,
            self.review_state,
        )
        .metadata(json!({
            "engine": "decision",
            "candidate_kind": self.kind.as_str(),
        }));

        if let (Some(kind), Some(id)) = (self.decided_by_entity_kind, &self.decided_by_entity_id) {
            decision = decision.decided_by(kind, id.clone());
        }

        let evidence =
            NewDecisionEvidence::new(self.evidence_source_kind, self.evidence_source_id.clone())
                .with_observation_id(self.evidence_observation_id.clone())
                .quote(self.quote.clone())
                .confidence(self.confidence)
                .metadata(json!({
                    "engine": "decision",
                    "candidate_kind": self.kind.as_str(),
                }));

        let impacted_entities = self
            .impacted_entities
            .iter()
            .map(|entity| {
                NewDecisionImpactedEntity::new(entity.entity_kind, entity.entity_id.clone())
                    .impact_type(entity.impact_type.clone())
                    .metadata(json!({
                        "engine": "decision",
                        "candidate_kind": self.kind.as_str(),
                    }))
            })
            .collect();

        (decision, evidence, impacted_entities)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecisionImpactedEntityCandidate {
    pub entity_kind: DecisionEntityKind,
    pub entity_id: String,
    pub impact_type: String,
}

pub fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), DecisionEngineError> {
    if value.trim().is_empty() {
        return Err(DecisionEngineError::EmptyField(field_name));
    }

    Ok(())
}
