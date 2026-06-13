use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::domains::decisions::{
    DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, NewDecision,
    NewDecisionEvidence, NewDecisionImpactedEntity,
};

pub struct DecisionEngine;

impl DecisionEngine {
    pub fn detect_candidates(
        input: &DecisionExtractionInput,
    ) -> Result<DecisionExtractionResult, DecisionEngineError> {
        input.validate()?;

        let mut result = DecisionExtractionResult::default();
        for sentence in sentences(&input.text) {
            if let Some(candidate) = detect_decision(input, sentence) {
                result.decisions.push(candidate);
            }
        }

        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionExtractionInput {
    pub source_kind: DecisionEvidenceSourceKind,
    pub source_id: String,
    pub text: String,
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

    fn validate(&self) -> Result<(), DecisionEngineError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("impacted_entity_id", &self.impacted_entity_id)?;
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

#[derive(Debug, Error)]
pub enum DecisionEngineError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("decided_by entity kind and id must be provided together")]
    PartialDecider,
}

fn detect_decision(input: &DecisionExtractionInput, sentence: &str) -> Option<DecisionCandidate> {
    let normalized_sentence = sentence.trim();
    let lower = normalized_sentence.to_lowercase();

    let (kind, body_start, confidence) = if lower.starts_with("decision:") {
        (
            DecisionCandidateKind::ExplicitDecision,
            "decision:".len(),
            0.83,
        )
    } else if lower.starts_with("we decided to ") {
        (
            DecisionCandidateKind::ExplicitDecision,
            "we decided to ".len(),
            0.78,
        )
    } else if lower.starts_with("approved:") {
        (DecisionCandidateKind::Approval, "approved:".len(), 0.74)
    } else if lower.starts_with("confirmed:") {
        (
            DecisionCandidateKind::Confirmation,
            "confirmed:".len(),
            0.72,
        )
    } else {
        return None;
    };

    let body = normalized_sentence[body_start..]
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .trim();
    if body.is_empty() {
        return None;
    }

    let (title, rationale) = split_rationale(body);
    if title.len() < 3 || rationale.len() < 3 {
        return None;
    }

    Some(DecisionCandidate {
        kind,
        title,
        rationale,
        quote: ensure_sentence_terminator(normalized_sentence),
        confidence,
        review_state: DecisionReviewState::Suggested,
        evidence_source_kind: input.source_kind,
        evidence_source_id: input.source_id.clone(),
        decided_by_entity_kind: input.decided_by_entity_kind,
        decided_by_entity_id: input.decided_by_entity_id.clone(),
        impacted_entities: vec![DecisionImpactedEntityCandidate {
            entity_kind: input.impacted_entity_kind,
            entity_id: input.impacted_entity_id.clone(),
            impact_type: "decision_context".to_owned(),
        }],
    })
}

fn split_rationale(value: &str) -> (String, String) {
    let lower = value.to_lowercase();
    for marker in [" because ", " so that "] {
        if let Some(index) = lower.find(marker) {
            let title = value[..index].trim().to_owned();
            let rationale = value[index + marker.len()..].trim().to_owned();
            if !title.is_empty() && !rationale.is_empty() {
                return (title, rationale);
            }
        }
    }

    (value.trim().to_owned(), value.trim().to_owned())
}

fn sentences(text: &str) -> Vec<&str> {
    text.split_terminator(['\n', '.', '!', '?'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

fn ensure_sentence_terminator(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.ends_with(['.', '!', '?']) {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.")
    }
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), DecisionEngineError> {
    if value.trim().is_empty() {
        return Err(DecisionEngineError::EmptyField(field_name));
    }

    Ok(())
}
