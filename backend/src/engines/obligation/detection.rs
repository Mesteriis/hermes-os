use super::errors::ObligationEngineError;
use super::models::{
    ObligationCandidate, ObligationCandidateKind, ObligationExtractionInput, ObligationReviewState,
    validate_non_empty,
};

pub(crate) fn detect_commitment(
    input: &ObligationExtractionInput,
    sentence: &str,
) -> Option<ObligationCandidate> {
    let normalized_sentence = sentence.trim();
    let lower = normalized_sentence.to_lowercase();

    let (kind, statement_start, confidence) = if lower.starts_with("i will ") {
        (ObligationCandidateKind::Commitment, "i will ".len(), 0.84)
    } else if lower.starts_with("i'll ") {
        (ObligationCandidateKind::Commitment, "i'll ".len(), 0.84)
    } else if lower.starts_with("please ") {
        (ObligationCandidateKind::Request, "please ".len(), 0.76)
    } else {
        return None;
    };

    let body = normalized_sentence[statement_start..]
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .trim();
    if body.is_empty() {
        return None;
    }

    let (statement, due_text) = split_due_text(body);
    let (statement, condition) = split_condition(&statement);
    let statement = statement.trim();
    if statement.len() < 3 {
        return None;
    }

    Some(ObligationCandidate {
        kind,
        obligated_entity_kind: input.obligated_entity_kind,
        obligated_entity_id: input.obligated_entity_id.clone(),
        beneficiary_entity_kind: input.beneficiary_entity_kind,
        beneficiary_entity_id: input.beneficiary_entity_id.clone(),
        statement: statement.to_owned(),
        quote: ensure_sentence_terminator(normalized_sentence),
        due_text,
        condition,
        confidence,
        review_state: ObligationReviewState::Suggested,
        evidence_source_kind: input.source_kind,
        evidence_source_id: input.source_id.clone(),
    })
}

fn split_due_text(value: &str) -> (String, Option<String>) {
    let lower = value.to_lowercase();
    for marker in [" by ", " before "] {
        if let Some(index) = lower.find(marker) {
            let statement = value[..index].trim().to_owned();
            let due_text = value[index + marker.len()..]
                .trim()
                .trim_end_matches(['.', '!', '?'])
                .trim()
                .to_owned();
            if !due_text.is_empty() {
                return (statement, Some(due_text));
            }
        }
    }

    (value.trim().to_owned(), None)
}

fn split_condition(value: &str) -> (String, Option<String>) {
    let lower = value.to_lowercase();
    for marker in [" when ", " once ", " if "] {
        if let Some(index) = lower.find(marker) {
            let statement = value[..index].trim().to_owned();
            let condition = value[index + marker.len()..]
                .trim()
                .trim_end_matches(['.', '!', '?'])
                .trim()
                .to_owned();
            if !condition.is_empty() {
                return (statement, Some(condition));
            }
        }
    }

    (value.trim().to_owned(), None)
}

pub(crate) fn sentences(text: &str) -> Vec<&str> {
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
