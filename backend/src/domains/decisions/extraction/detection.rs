use super::models::{
    DecisionCandidate, DecisionCandidateKind, DecisionExtractionInput,
    DecisionImpactedEntityCandidate,
};
use crate::domains::decisions::models::states::DecisionReviewState;

pub fn detect_decision(
    input: &DecisionExtractionInput,
    sentence: &str,
) -> Option<DecisionCandidate> {
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
        evidence_observation_id: input.observation_id.clone(),
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

pub fn sentences(text: &str) -> Vec<&str> {
    text.split_terminator(['\n', '.', '!', '?'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

pub fn ensure_sentence_terminator(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.ends_with(['.', '!', '?']) {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.")
    }
}
