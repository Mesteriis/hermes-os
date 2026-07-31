use std::collections::BTreeSet;

use crate::{
    COMMUNICATION_TASK_MAX_CANDIDATES_V1, COMMUNICATION_TASK_MAX_CONFIDENCE_BASIS_POINTS_V1,
    COMMUNICATION_TASK_MAX_HINT_CHARS_V1, COMMUNICATION_TASK_MAX_TITLE_CHARS_V1,
    CommunicationTaskCandidateCompletenessV1, CommunicationTaskCandidateRejectionCodeV1,
    CommunicationTaskCandidateStateV1, CommunicationTaskCandidateStatusV1,
    CommunicationTaskCandidateV1, model::zero,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateTransitionV1 {
    BeginSourcePreparation,
    SourcePrepared {
        source_evidence_id: [u8; 16],
        source_evidence_revision: u64,
        source_sha256: [u8; 32],
    },
    Complete {
        source_sha256: [u8; 32],
        candidates: Vec<CommunicationTaskCandidateV1>,
    },
    Reject(CommunicationTaskCandidateRejectionCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateTransitionErrorV1 {
    InvalidTransition,
    InvalidSourceReceipt,
    InvalidCandidates,
    DuplicateCandidateId,
    SourceIdentityMismatch,
    SourceDigestMismatch,
    RevisionExhausted,
}

#[must_use]
pub fn accepted_communication_task_candidate_status_v1() -> CommunicationTaskCandidateStatusV1 {
    CommunicationTaskCandidateStatusV1 {
        state: CommunicationTaskCandidateStateV1::Accepted,
        state_revision: 1,
        source_evidence_id: None,
        source_evidence_revision: None,
        source_sha256: None,
        candidates: None,
        completeness: None,
        rejection: None,
    }
}

pub fn transition_communication_task_candidate_v1(
    current: &CommunicationTaskCandidateStatusV1,
    transition: CommunicationTaskCandidateTransitionV1,
) -> Result<CommunicationTaskCandidateStatusV1, CommunicationTaskCandidateTransitionErrorV1> {
    let next_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(CommunicationTaskCandidateTransitionErrorV1::RevisionExhausted)?;
    let next = match (current.state, transition) {
        (
            CommunicationTaskCandidateStateV1::Accepted,
            CommunicationTaskCandidateTransitionV1::BeginSourcePreparation,
        ) => CommunicationTaskCandidateStatusV1 {
            state: CommunicationTaskCandidateStateV1::PreparingSource,
            state_revision: next_revision,
            ..current.clone()
        },
        (
            CommunicationTaskCandidateStateV1::PreparingSource,
            CommunicationTaskCandidateTransitionV1::SourcePrepared {
                source_evidence_id,
                source_evidence_revision,
                source_sha256,
            },
        ) => {
            if zero(&source_evidence_id) || source_evidence_revision == 0 || zero(&source_sha256) {
                return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidSourceReceipt);
            }
            CommunicationTaskCandidateStatusV1 {
                state: CommunicationTaskCandidateStateV1::Extracting,
                state_revision: next_revision,
                source_evidence_id: Some(source_evidence_id),
                source_evidence_revision: Some(source_evidence_revision),
                source_sha256: Some(source_sha256),
                candidates: None,
                completeness: None,
                rejection: None,
            }
        }
        (
            CommunicationTaskCandidateStateV1::Extracting,
            CommunicationTaskCandidateTransitionV1::Complete {
                source_sha256,
                candidates,
            },
        ) => {
            if current.source_sha256 != Some(source_sha256) {
                return Err(CommunicationTaskCandidateTransitionErrorV1::SourceDigestMismatch);
            }
            validate_candidates(current, &candidates)?;
            CommunicationTaskCandidateStatusV1 {
                state: CommunicationTaskCandidateStateV1::Ready,
                state_revision: next_revision,
                candidates: Some(candidates),
                completeness: Some(CommunicationTaskCandidateCompletenessV1::Complete),
                rejection: None,
                ..current.clone()
            }
        }
        (
            CommunicationTaskCandidateStateV1::Accepted
            | CommunicationTaskCandidateStateV1::PreparingSource
            | CommunicationTaskCandidateStateV1::Extracting,
            CommunicationTaskCandidateTransitionV1::Reject(rejection),
        ) => CommunicationTaskCandidateStatusV1 {
            state: CommunicationTaskCandidateStateV1::Rejected,
            state_revision: next_revision,
            candidates: None,
            completeness: None,
            rejection: Some(rejection),
            ..current.clone()
        },
        _ => return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidTransition),
    };
    validate_communication_task_candidate_status_v1(&next)?;
    Ok(next)
}

pub fn validate_communication_task_candidate_status_v1(
    status: &CommunicationTaskCandidateStatusV1,
) -> Result<(), CommunicationTaskCandidateTransitionErrorV1> {
    if status.state_revision == 0 {
        return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidTransition);
    }
    let source_absent = status.source_evidence_id.is_none()
        && status.source_evidence_revision.is_none()
        && status.source_sha256.is_none();
    let source_present = status.source_evidence_id.is_some_and(|value| !zero(&value))
        && status
            .source_evidence_revision
            .is_some_and(|value| value > 0)
        && status.source_sha256.is_some_and(|value| !zero(&value));
    match status.state {
        CommunicationTaskCandidateStateV1::Accepted
        | CommunicationTaskCandidateStateV1::PreparingSource => {
            if !source_absent
                || status.candidates.is_some()
                || status.completeness.is_some()
                || status.rejection.is_some()
            {
                return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidTransition);
            }
        }
        CommunicationTaskCandidateStateV1::Extracting => {
            if !source_present
                || status.candidates.is_some()
                || status.completeness.is_some()
                || status.rejection.is_some()
            {
                return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidTransition);
            }
        }
        CommunicationTaskCandidateStateV1::Ready => {
            let candidates = status
                .candidates
                .as_deref()
                .ok_or(CommunicationTaskCandidateTransitionErrorV1::InvalidCandidates)?;
            if !source_present
                || status.completeness != Some(CommunicationTaskCandidateCompletenessV1::Complete)
                || status.rejection.is_some()
            {
                return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidTransition);
            }
            validate_candidates(status, candidates)?;
        }
        CommunicationTaskCandidateStateV1::Rejected => {
            if (!source_absent && !source_present)
                || status.candidates.is_some()
                || status.completeness.is_some()
                || status.rejection.is_none()
            {
                return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidTransition);
            }
        }
    }
    Ok(())
}

fn validate_candidates(
    status: &CommunicationTaskCandidateStatusV1,
    candidates: &[CommunicationTaskCandidateV1],
) -> Result<(), CommunicationTaskCandidateTransitionErrorV1> {
    if candidates.len() > COMMUNICATION_TASK_MAX_CANDIDATES_V1 {
        return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidCandidates);
    }
    let mut ids = BTreeSet::new();
    for candidate in candidates {
        if zero(&candidate.candidate_id)
            || zero(&candidate.candidate_digest)
            || candidate.title.is_empty()
            || candidate.title.chars().count() > COMMUNICATION_TASK_MAX_TITLE_CHARS_V1
            || candidate.due_text_hint.as_ref().is_some_and(|value| {
                value.is_empty() || value.chars().count() > COMMUNICATION_TASK_MAX_HINT_CHARS_V1
            })
            || candidate.assignee_label_hint.as_ref().is_some_and(|value| {
                value.is_empty() || value.chars().count() > COMMUNICATION_TASK_MAX_HINT_CHARS_V1
            })
            || !(1..=COMMUNICATION_TASK_MAX_CONFIDENCE_BASIS_POINTS_V1)
                .contains(&candidate.confidence_basis_points)
        {
            return Err(CommunicationTaskCandidateTransitionErrorV1::InvalidCandidates);
        }
        if !ids.insert(candidate.candidate_id) {
            return Err(CommunicationTaskCandidateTransitionErrorV1::DuplicateCandidateId);
        }
        if status.source_evidence_id != Some(candidate.source_evidence_id)
            || status.source_evidence_revision != Some(candidate.source_evidence_revision)
        {
            return Err(CommunicationTaskCandidateTransitionErrorV1::SourceIdentityMismatch);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{CommunicationTaskSignalKindV1, CommunicationTaskSourceBasisV1};

    use super::*;

    #[test]
    fn lifecycle_allows_empty_ready_result_without_creating_a_task() {
        let accepted = accepted_communication_task_candidate_status_v1();
        let preparing = transition_communication_task_candidate_v1(
            &accepted,
            CommunicationTaskCandidateTransitionV1::BeginSourcePreparation,
        )
        .expect("prepare");
        let extracting = transition_communication_task_candidate_v1(
            &preparing,
            CommunicationTaskCandidateTransitionV1::SourcePrepared {
                source_evidence_id: [3; 16],
                source_evidence_revision: 7,
                source_sha256: [4; 32],
            },
        )
        .expect("source");
        let ready = transition_communication_task_candidate_v1(
            &extracting,
            CommunicationTaskCandidateTransitionV1::Complete {
                source_sha256: [4; 32],
                candidates: Vec::new(),
            },
        )
        .expect("complete");
        assert_eq!(ready.state, CommunicationTaskCandidateStateV1::Ready);
        assert_eq!(ready.candidates, Some(Vec::new()));
    }

    #[test]
    fn lifecycle_rejects_candidate_from_another_source() {
        let status = CommunicationTaskCandidateStatusV1 {
            state: CommunicationTaskCandidateStateV1::Extracting,
            state_revision: 3,
            source_evidence_id: Some([3; 16]),
            source_evidence_revision: Some(7),
            source_sha256: Some([4; 32]),
            candidates: None,
            completeness: None,
            rejection: None,
        };
        let candidate = CommunicationTaskCandidateV1 {
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            title: "Action: send report".to_owned(),
            due_text_hint: None,
            assignee_label_hint: None,
            source_basis: CommunicationTaskSourceBasisV1::Body,
            signal_kind: CommunicationTaskSignalKindV1::ExplicitAction,
            confidence_basis_points: 9_000,
            source_evidence_id: [9; 16],
            source_evidence_revision: 7,
        };
        assert_eq!(
            transition_communication_task_candidate_v1(
                &status,
                CommunicationTaskCandidateTransitionV1::Complete {
                    source_sha256: [4; 32],
                    candidates: vec![candidate],
                },
            ),
            Err(CommunicationTaskCandidateTransitionErrorV1::SourceIdentityMismatch)
        );
    }
}
