use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationEvidenceSourceKind,
    ObligationReviewState,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ObligationExtractionInput {
    pub source_kind: ObligationEvidenceSourceKind,
    pub source_id: String,
    pub text: String,
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
}

impl ObligationExtractionInput {
    pub fn communication(
        source_id: impl Into<String>,
        text: impl Into<String>,
        obligated_entity_kind: ObligationEntityKind,
        obligated_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_kind: ObligationEvidenceSourceKind::Communication,
            source_id: source_id.into(),
            text: text.into(),
            obligated_entity_kind,
            obligated_entity_id: obligated_entity_id.into(),
            beneficiary_entity_kind: None,
            beneficiary_entity_id: None,
        }
    }

    pub fn document(
        source_id: impl Into<String>,
        text: impl Into<String>,
        obligated_entity_kind: ObligationEntityKind,
        obligated_entity_id: impl Into<String>,
    ) -> Self {
        Self {
            source_kind: ObligationEvidenceSourceKind::Document,
            source_id: source_id.into(),
            text: text.into(),
            obligated_entity_kind,
            obligated_entity_id: obligated_entity_id.into(),
            beneficiary_entity_kind: None,
            beneficiary_entity_id: None,
        }
    }

    pub fn beneficiary(
        mut self,
        beneficiary_entity_kind: ObligationEntityKind,
        beneficiary_entity_id: impl Into<String>,
    ) -> Self {
        self.beneficiary_entity_kind = Some(beneficiary_entity_kind);
        self.beneficiary_entity_id = Some(beneficiary_entity_id.into());
        self
    }

    pub(crate) fn validate(
        &self,
    ) -> Result<(), crate::engines::obligation::errors::ObligationEngineError> {
        use crate::engines::obligation::errors::ObligationEngineError;
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("obligated_entity_id", &self.obligated_entity_id)?;
        match (
            self.beneficiary_entity_kind,
            self.beneficiary_entity_id.as_ref(),
        ) {
            (None, None) => {}
            (Some(_), Some(beneficiary_entity_id)) => {
                validate_non_empty("beneficiary_entity_id", beneficiary_entity_id)?;
            }
            _ => return Err(ObligationEngineError::PartialBeneficiary),
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObligationExtractionResult {
    pub obligations: Vec<ObligationCandidate>,
    pub task_candidates: Vec<ObligationTaskCandidate>,
    pub follow_ups: Vec<FollowUpCandidate>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationCandidateKind {
    Commitment,
    Request,
}

impl ObligationCandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Commitment => "commitment",
            Self::Request => "request",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObligationCandidate {
    pub kind: ObligationCandidateKind,
    pub obligated_entity_kind: ObligationEntityKind,
    pub obligated_entity_id: String,
    pub beneficiary_entity_kind: Option<ObligationEntityKind>,
    pub beneficiary_entity_id: Option<String>,
    pub statement: String,
    pub quote: String,
    pub due_text: Option<String>,
    pub condition: Option<String>,
    pub confidence: f64,
    pub review_state: ObligationReviewState,
    pub evidence_source_kind: ObligationEvidenceSourceKind,
    pub evidence_source_id: String,
}

impl ObligationCandidate {
    pub fn to_obligation_draft(&self) -> (NewObligation, NewObligationEvidence) {
        let mut obligation = NewObligation::new(
            self.obligated_entity_kind,
            self.obligated_entity_id.clone(),
            self.statement.clone(),
            self.confidence,
            self.review_state,
        )
        .metadata(serde_json::json!({
            "engine": "obligation",
            "candidate_kind": self.kind.as_str(),
            "due_text": self.due_text,
            "condition": self.condition,
        }));

        if let (Some(kind), Some(id)) = (self.beneficiary_entity_kind, &self.beneficiary_entity_id)
        {
            obligation = obligation.beneficiary(kind, id.clone());
        }
        if let Some(condition) = &self.condition {
            obligation = obligation.condition(condition.clone());
        }

        let evidence =
            NewObligationEvidence::new(self.evidence_source_kind, self.evidence_source_id.clone())
                .quote(self.quote.clone())
                .confidence(self.confidence)
                .metadata(serde_json::json!({
                    "engine": "obligation",
                    "candidate_kind": self.kind.as_str(),
                }));

        (obligation, evidence)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObligationTaskCandidate {
    pub source_obligation_statement: String,
    pub statement: String,
    pub suggested_title: String,
    pub due_text: Option<String>,
    pub confidence: f64,
}

impl ObligationTaskCandidate {
    pub(crate) fn from_obligation(candidate: &ObligationCandidate) -> Self {
        Self {
            source_obligation_statement: candidate.statement.clone(),
            statement: candidate.statement.clone(),
            suggested_title: candidate.statement.clone(),
            due_text: candidate.due_text.clone(),
            confidence: (candidate.confidence - 0.08).max(0.0),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FollowUpCandidate {
    pub source_obligation_statement: String,
    pub prompt: String,
    pub due_text: Option<String>,
    pub confidence: f64,
}

impl FollowUpCandidate {
    pub(crate) fn from_obligation(candidate: &ObligationCandidate) -> Self {
        Self {
            source_obligation_statement: candidate.statement.clone(),
            prompt: format!("Follow up on: {}", candidate.statement),
            due_text: candidate.due_text.clone(),
            confidence: (candidate.confidence - 0.12).max(0.0),
        }
    }
}

pub(crate) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), crate::engines::obligation::errors::ObligationEngineError> {
    use crate::engines::obligation::errors::ObligationEngineError;
    if value.trim().is_empty() {
        return Err(ObligationEngineError::EmptyField(field_name));
    }
    Ok(())
}
