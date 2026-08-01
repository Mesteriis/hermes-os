use std::collections::BTreeSet;

use crate::{
    COMMUNICATION_NOTE_MAX_CANDIDATES_V1, COMMUNICATION_NOTE_MAX_CONFIDENCE_BASIS_POINTS_V1,
    COMMUNICATION_NOTE_MAX_EXCERPT_CHARS_V1, COMMUNICATION_NOTE_MAX_TITLE_CHARS_V1,
    CommunicationNoteCandidateCompletenessV1, CommunicationNoteCandidateRejectionCodeV1,
    CommunicationNoteCandidateStateV1, CommunicationNoteCandidateStatusV1,
    CommunicationNoteCandidateV1, model::zero,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateTransitionV1 {
    BeginSourcePreparation,
    SourcePrepared {
        source_evidence_id: [u8; 16],
        source_evidence_revision: u64,
        source_sha256: [u8; 32],
    },
    Complete {
        source_sha256: [u8; 32],
        candidates: Vec<CommunicationNoteCandidateV1>,
    },
    Reject(CommunicationNoteCandidateRejectionCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateTransitionErrorV1 {
    InvalidTransition,
    InvalidSourceReceipt,
    InvalidCandidates,
    DuplicateCandidateId,
    SourceIdentityMismatch,
    SourceDigestMismatch,
    RevisionExhausted,
}

#[must_use]
pub fn accepted_communication_note_candidate_status_v1() -> CommunicationNoteCandidateStatusV1 {
    CommunicationNoteCandidateStatusV1 {
        state: CommunicationNoteCandidateStateV1::Accepted,
        state_revision: 1,
        source_evidence_id: None,
        source_evidence_revision: None,
        source_sha256: None,
        candidates: None,
        completeness: None,
        rejection: None,
    }
}

pub fn transition_communication_note_candidate_v1(
    current: &CommunicationNoteCandidateStatusV1,
    transition: CommunicationNoteCandidateTransitionV1,
) -> Result<CommunicationNoteCandidateStatusV1, CommunicationNoteCandidateTransitionErrorV1> {
    let next_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(CommunicationNoteCandidateTransitionErrorV1::RevisionExhausted)?;
    let next = match (current.state, transition) {
        (
            CommunicationNoteCandidateStateV1::Accepted,
            CommunicationNoteCandidateTransitionV1::BeginSourcePreparation,
        ) => CommunicationNoteCandidateStatusV1 {
            state: CommunicationNoteCandidateStateV1::PreparingSource,
            state_revision: next_revision,
            ..current.clone()
        },
        (
            CommunicationNoteCandidateStateV1::PreparingSource,
            CommunicationNoteCandidateTransitionV1::SourcePrepared {
                source_evidence_id,
                source_evidence_revision,
                source_sha256,
            },
        ) => {
            if zero(&source_evidence_id) || source_evidence_revision == 0 || zero(&source_sha256) {
                return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidSourceReceipt);
            }
            CommunicationNoteCandidateStatusV1 {
                state: CommunicationNoteCandidateStateV1::Extracting,
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
            CommunicationNoteCandidateStateV1::Extracting,
            CommunicationNoteCandidateTransitionV1::Complete {
                source_sha256,
                candidates,
            },
        ) => {
            if current.source_sha256 != Some(source_sha256) {
                return Err(CommunicationNoteCandidateTransitionErrorV1::SourceDigestMismatch);
            }
            validate_candidates(current, &candidates)?;
            CommunicationNoteCandidateStatusV1 {
                state: CommunicationNoteCandidateStateV1::Ready,
                state_revision: next_revision,
                candidates: Some(candidates),
                completeness: Some(CommunicationNoteCandidateCompletenessV1::Complete),
                rejection: None,
                ..current.clone()
            }
        }
        (
            CommunicationNoteCandidateStateV1::Accepted
            | CommunicationNoteCandidateStateV1::PreparingSource
            | CommunicationNoteCandidateStateV1::Extracting,
            CommunicationNoteCandidateTransitionV1::Reject(rejection),
        ) => CommunicationNoteCandidateStatusV1 {
            state: CommunicationNoteCandidateStateV1::Rejected,
            state_revision: next_revision,
            candidates: None,
            completeness: None,
            rejection: Some(rejection),
            ..current.clone()
        },
        _ => return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidTransition),
    };
    validate_communication_note_candidate_status_v1(&next)?;
    Ok(next)
}

pub fn validate_communication_note_candidate_status_v1(
    status: &CommunicationNoteCandidateStatusV1,
) -> Result<(), CommunicationNoteCandidateTransitionErrorV1> {
    if status.state_revision == 0 {
        return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidTransition);
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
        CommunicationNoteCandidateStateV1::Accepted
        | CommunicationNoteCandidateStateV1::PreparingSource => {
            if !source_absent
                || status.candidates.is_some()
                || status.completeness.is_some()
                || status.rejection.is_some()
            {
                return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidTransition);
            }
        }
        CommunicationNoteCandidateStateV1::Extracting => {
            if !source_present
                || status.candidates.is_some()
                || status.completeness.is_some()
                || status.rejection.is_some()
            {
                return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidTransition);
            }
        }
        CommunicationNoteCandidateStateV1::Ready => {
            let candidates = status
                .candidates
                .as_deref()
                .ok_or(CommunicationNoteCandidateTransitionErrorV1::InvalidCandidates)?;
            if !source_present
                || status.completeness != Some(CommunicationNoteCandidateCompletenessV1::Complete)
                || status.rejection.is_some()
            {
                return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidTransition);
            }
            validate_candidates(status, candidates)?;
        }
        CommunicationNoteCandidateStateV1::Rejected => {
            if (!source_absent && !source_present)
                || status.candidates.is_some()
                || status.completeness.is_some()
                || status.rejection.is_none()
            {
                return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidTransition);
            }
        }
    }
    Ok(())
}

fn validate_candidates(
    status: &CommunicationNoteCandidateStatusV1,
    candidates: &[CommunicationNoteCandidateV1],
) -> Result<(), CommunicationNoteCandidateTransitionErrorV1> {
    if candidates.len() > COMMUNICATION_NOTE_MAX_CANDIDATES_V1 {
        return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidCandidates);
    }
    let mut ids = BTreeSet::new();
    for candidate in candidates {
        if zero(&candidate.candidate_id)
            || zero(&candidate.candidate_digest)
            || candidate.title.is_empty()
            || candidate.title.chars().count() > COMMUNICATION_NOTE_MAX_TITLE_CHARS_V1
            || candidate.excerpt.chars().count() > COMMUNICATION_NOTE_MAX_EXCERPT_CHARS_V1
            || candidate.topic_hints.is_empty()
            || candidate.topic_hints.len() > 4
            || candidate
                .topic_hints
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || !(1..=COMMUNICATION_NOTE_MAX_CONFIDENCE_BASIS_POINTS_V1)
                .contains(&candidate.confidence_basis_points)
        {
            return Err(CommunicationNoteCandidateTransitionErrorV1::InvalidCandidates);
        }
        if !ids.insert(candidate.candidate_id) {
            return Err(CommunicationNoteCandidateTransitionErrorV1::DuplicateCandidateId);
        }
        if status.source_evidence_id != Some(candidate.source_evidence_id)
            || status.source_evidence_revision != Some(candidate.source_evidence_revision)
        {
            return Err(CommunicationNoteCandidateTransitionErrorV1::SourceIdentityMismatch);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{CommunicationNoteSourceBasisV1, CommunicationNoteTopicHintV1};

    use super::*;

    #[test]
    fn lifecycle_allows_empty_ready_result_without_creating_a_note() {
        let accepted = accepted_communication_note_candidate_status_v1();
        let preparing = transition_communication_note_candidate_v1(
            &accepted,
            CommunicationNoteCandidateTransitionV1::BeginSourcePreparation,
        )
        .expect("prepare");
        let extracting = transition_communication_note_candidate_v1(
            &preparing,
            CommunicationNoteCandidateTransitionV1::SourcePrepared {
                source_evidence_id: [3; 16],
                source_evidence_revision: 7,
                source_sha256: [4; 32],
            },
        )
        .expect("source");
        let ready = transition_communication_note_candidate_v1(
            &extracting,
            CommunicationNoteCandidateTransitionV1::Complete {
                source_sha256: [4; 32],
                candidates: Vec::new(),
            },
        )
        .expect("complete");
        assert_eq!(ready.state, CommunicationNoteCandidateStateV1::Ready);
        assert_eq!(ready.candidates, Some(Vec::new()));
    }

    #[test]
    fn lifecycle_rejects_candidate_from_another_source() {
        let status = CommunicationNoteCandidateStatusV1 {
            state: CommunicationNoteCandidateStateV1::Extracting,
            state_revision: 3,
            source_evidence_id: Some([3; 16]),
            source_evidence_revision: Some(7),
            source_sha256: Some([4; 32]),
            candidates: None,
            completeness: None,
            rejection: None,
        };
        let candidate = CommunicationNoteCandidateV1 {
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            title: "Contract approved".to_owned(),
            excerpt: "The agreement was approved.".to_owned(),
            topic_hints: vec![CommunicationNoteTopicHintV1::Legal],
            source_basis: CommunicationNoteSourceBasisV1::Body,
            confidence_basis_points: 9_000,
            source_evidence_id: [9; 16],
            source_evidence_revision: 7,
        };
        assert_eq!(
            transition_communication_note_candidate_v1(
                &status,
                CommunicationNoteCandidateTransitionV1::Complete {
                    source_sha256: [4; 32],
                    candidates: vec![candidate],
                },
            ),
            Err(CommunicationNoteCandidateTransitionErrorV1::SourceIdentityMismatch)
        );
    }
}
