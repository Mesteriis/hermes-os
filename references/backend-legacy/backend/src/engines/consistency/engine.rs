use serde_json::json;

use super::errors::ConsistencyError;
use super::helpers::{
    claim_text, contradiction_metadata, normalize_claim_value, severity_for_confidence,
};
use super::models::{
    AcceptedClaim, ContradictionReviewState, EvidenceClaimExtractionInput,
    NewContradictionObservation, NewEvidenceClaim,
};
use super::parsing::parse_evidence_claim_line;

pub struct ConsistencyEngine;

impl ConsistencyEngine {
    pub fn detect_claim_contradictions(
        accepted_claims: &[AcceptedClaim],
        new_claims: &[NewEvidenceClaim],
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        Self::detect_claim_contradictions_with_detector(
            accepted_claims,
            new_claims,
            "structured_claim",
        )
    }

    pub fn extract_evidence_claims(
        input: &EvidenceClaimExtractionInput,
    ) -> Result<Vec<NewEvidenceClaim>, ConsistencyError> {
        input.validate()?;

        let mut claims = Vec::new();
        for line in input.text.lines() {
            let Some((claim_type, value)) = parse_evidence_claim_line(line) else {
                continue;
            };

            claims.push(NewEvidenceClaim {
                subject_id: input.subject_id.clone(),
                claim_type,
                value,
                source_kind: input.source_kind,
                source_id: input.source_id.clone(),
                confidence: input.confidence,
            });
        }

        Ok(claims)
    }

    pub fn detect_evidence_contradictions(
        accepted_claims: &[AcceptedClaim],
        evidence_inputs: &[EvidenceClaimExtractionInput],
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        let mut extracted_claims = Vec::new();
        for input in evidence_inputs {
            extracted_claims.extend(Self::extract_evidence_claims(input)?);
        }

        Self::detect_claim_contradictions_with_detector(
            accepted_claims,
            &extracted_claims,
            "structured_evidence_claim",
        )
    }

    fn detect_claim_contradictions_with_detector(
        accepted_claims: &[AcceptedClaim],
        new_claims: &[NewEvidenceClaim],
        detector: &str,
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        for claim in accepted_claims {
            claim.validate()?;
        }
        for claim in new_claims {
            claim.validate()?;
        }

        let mut observations = Vec::new();
        for accepted in accepted_claims {
            for new_claim in new_claims {
                if accepted.subject_id != new_claim.subject_id {
                    continue;
                }
                if accepted.claim_type.trim() != new_claim.claim_type.trim() {
                    continue;
                }
                if normalize_claim_value(&accepted.value) == normalize_claim_value(&new_claim.value)
                {
                    continue;
                }

                let confidence = accepted.confidence.min(new_claim.confidence);
                observations.push(NewContradictionObservation {
                    old_source_kind: accepted.source_kind,
                    old_source_id: accepted.source_id.clone(),
                    new_source_kind: new_claim.source_kind,
                    new_source_id: new_claim.source_id.clone(),
                    affected_entities: json!([{
                        "entity_kind": "subject",
                        "entity_id": accepted.subject_id,
                    }]),
                    conflict_type: "direct_contradiction".to_owned(),
                    old_claim: claim_text(&accepted.claim_type, &accepted.value),
                    new_claim: claim_text(&new_claim.claim_type, &new_claim.value),
                    confidence,
                    severity: severity_for_confidence(confidence),
                    review_state: ContradictionReviewState::Suggested,
                    metadata: contradiction_metadata(detector, accepted, new_claim),
                });
            }
        }

        Ok(observations)
    }
}
