use serde_json::json;

use crate::domains::obligations::ObligationReviewState;
use crate::engines::obligation::ObligationCandidate;

use super::constants::{
    OBLIGATION_CANDIDATE_METADATA_KEY, REVIEW_TEXT_SNIPPET_CHARS, TITLE_PREVIEW_CHARS,
};
use super::errors::TaskCandidateError;
use super::models::{
    CandidatePayload, StoredCandidateRow, TaskCandidateKind, TaskCandidateReviewState,
    TaskCandidateSourceKind,
};
use super::validation::text_preview;

pub(crate) struct CandidateFragment {
    pub(crate) text: String,
    pub(crate) due_text: Option<String>,
    pub(crate) assignee_label: Option<String>,
}

pub(crate) fn extract_candidate_fragment(text: &str) -> Option<CandidateFragment> {
    let text_lower = text.to_lowercase();
    if !(text_lower.contains("action:")
        || text_lower.contains("please ")
        || text_lower.contains("follow up")
        || text_lower.contains("next step"))
    {
        return None;
    }

    let selected = text
        .lines()
        .map(str::trim)
        .find(|line| {
            let lower = line.to_lowercase();
            lower.contains("action:")
                || lower.contains("please ")
                || lower.contains("follow up")
                || lower.contains("next step")
        })
        .unwrap_or(text);

    let due_text = text.lines().find_map(parse_due_text);
    let assignee_label = text.lines().find_map(parse_assignee_label);

    Some(CandidateFragment {
        text: selected.to_owned(),
        due_text,
        assignee_label,
    })
}

pub(crate) fn parse_due_text(line: &str) -> Option<String> {
    let normalized = line.trim().to_lowercase();
    if !normalized.starts_with("due") {
        return None;
    }

    normalized.split_once(':').and_then(|(_, right)| {
        let due = right.trim();
        (!due.is_empty()).then_some(due.to_owned())
    })
}

pub(crate) fn parse_assignee_label(line: &str) -> Option<String> {
    let normalized = line.to_lowercase();
    if !normalized.starts_with("assignee") {
        return None;
    }

    normalized.split_once(':').and_then(|(_, right)| {
        let assignee = right.trim();
        (!assignee.is_empty()).then_some(assignee.to_owned())
    })
}

pub(crate) fn title_from_fragment(value: &str) -> String {
    text_preview(value, TITLE_PREVIEW_CHARS)
}

pub(crate) fn evidence_excerpt(value: &str) -> String {
    text_preview(value, REVIEW_TEXT_SNIPPET_CHARS)
}

pub(crate) fn task_candidate_payload_from_obligation(
    observation_id: String,
    candidate: &ObligationCandidate,
) -> CandidatePayload {
    CandidatePayload {
        source_kind: TaskCandidateSourceKind::Observation,
        source_id: observation_id.clone(),
        observation_id: Some(observation_id),
        candidate_kind: TaskCandidateKind::ObligationTask,
        candidate_metadata: json!({
            "engine": "obligation",
            OBLIGATION_CANDIDATE_METADATA_KEY: candidate,
        }),
        project_id: None,
        title: title_from_fragment(&candidate.statement),
        due_text: candidate.due_text.clone(),
        assignee_label: None,
        confidence: (candidate.confidence - 0.08).max(0.0),
        evidence_excerpt: evidence_excerpt(&candidate.quote),
    }
}

pub(crate) fn obligation_candidate_from_metadata(
    candidate: &StoredCandidateRow,
) -> Result<ObligationCandidate, TaskCandidateError> {
    let value = candidate
        .candidate_metadata
        .get(OBLIGATION_CANDIDATE_METADATA_KEY)
        .cloned()
        .ok_or_else(|| {
            TaskCandidateError::InvalidCandidateMetadata(
                OBLIGATION_CANDIDATE_METADATA_KEY.to_owned(),
            )
        })?;

    Ok(serde_json::from_value(value)?)
}

pub(crate) fn obligation_review_state_from_task_candidate(
    review_state: TaskCandidateReviewState,
) -> ObligationReviewState {
    match review_state {
        TaskCandidateReviewState::Suggested => ObligationReviewState::Suggested,
        TaskCandidateReviewState::UserConfirmed => ObligationReviewState::UserConfirmed,
        TaskCandidateReviewState::UserRejected => ObligationReviewState::UserRejected,
    }
}
