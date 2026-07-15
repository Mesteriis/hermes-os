use serde_json::{Value, json};

use super::models::{
    AcceptedClaim, ContradictionSeverity, NewContradictionObservation, NewEvidenceClaim,
};

pub fn contradiction_observation_id(observation: &NewContradictionObservation) -> String {
    format!(
        "contradiction:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        observation.old_source_kind.as_str().len(),
        observation.old_source_kind.as_str(),
        observation.old_source_id.len(),
        observation.old_source_id,
        observation.new_source_kind.as_str().len(),
        observation.new_source_kind.as_str(),
        observation.new_source_id.len(),
        observation.new_source_id,
        observation.conflict_type.len(),
        observation.conflict_type
    )
}

pub(super) fn claim_text(claim_type: &str, value: &str) -> String {
    let claim_type = claim_type.trim();
    let value = value.trim();
    format!("{claim_type}={value}")
}

pub(super) fn normalize_claim_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(super) fn severity_for_confidence(confidence: f64) -> ContradictionSeverity {
    if confidence >= 0.95 {
        ContradictionSeverity::Critical
    } else if confidence >= 0.9 {
        ContradictionSeverity::High
    } else if confidence >= 0.7 {
        ContradictionSeverity::Medium
    } else {
        ContradictionSeverity::Low
    }
}

pub(super) fn contradiction_metadata(
    detector: &str,
    accepted: &AcceptedClaim,
    new_claim: &NewEvidenceClaim,
) -> Value {
    if detector == "structured_evidence_claim" {
        json!({
            "detector": detector,
            "claim_type": accepted.claim_type.trim(),
            "source_kind": new_claim.source_kind.as_str(),
        })
    } else {
        json!({
            "detector": detector,
            "claim_type": accepted.claim_type.trim(),
        })
    }
}
