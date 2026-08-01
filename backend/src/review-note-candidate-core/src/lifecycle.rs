use crate::model::{nonzero, valid_timestamp, validate_draft};
use crate::{
    ReviewNoteCandidateDecisionV1, ReviewNoteCandidateDraftV1,
    ReviewNoteCandidatePromotionResultV1, ReviewNoteCandidatePromotionStatusV1,
    ReviewNoteCandidateStateV1, ReviewNoteCandidateTimestampV1, ReviewNoteCandidateV1,
    ReviewNoteCandidateValidationErrorV1, STABLE_ID_BYTES_V1, derive_review_note_candidate_id_v1,
    validate_review_note_candidate_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidateTransitionErrorV1 {
    InvalidRecord,
    InvalidActor,
    InvalidTimestamp,
    RevisionConflict,
    TerminalDecision,
    PromotionNotPending,
    RevisionOverflow,
}

pub fn create_review_note_candidate_v1(
    draft: ReviewNoteCandidateDraftV1,
) -> Result<ReviewNoteCandidateV1, ReviewNoteCandidateTransitionErrorV1> {
    validate_draft(&draft).map_err(invalid_record)?;
    let review_id = derive_review_note_candidate_id_v1(
        &draft.logical_owner_id,
        &draft.candidate_id,
        &draft.candidate_digest,
    )
    .map_err(invalid_record)?;
    let review = ReviewNoteCandidateV1 {
        review_id,
        logical_owner_id: draft.logical_owner_id,
        candidate_id: draft.candidate_id,
        candidate_digest: draft.candidate_digest,
        source_evidence_id: draft.source_evidence_id,
        source_evidence_revision: draft.source_evidence_revision,
        title: draft.title,
        excerpt: draft.excerpt,
        topic_hints: draft.topic_hints,
        source_basis: draft.source_basis,
        confidence_basis_points: draft.confidence_basis_points,
        state: ReviewNoteCandidateStateV1::Pending,
        promotion_status: ReviewNoteCandidatePromotionStatusV1::NotRequested,
        review_revision: 1,
        decided_by_owner_device_id: None,
        decided_at: None,
        promoted_note_id: None,
        updated_at: draft.submitted_at,
    };
    validate_review_note_candidate_v1(&review).map_err(invalid_record)?;
    Ok(review)
}

pub fn decide_review_note_candidate_v1(
    current: &ReviewNoteCandidateV1,
    expected_review_revision: u64,
    decision: ReviewNoteCandidateDecisionV1,
    owner_device_id: [u8; STABLE_ID_BYTES_V1],
    decided_at: ReviewNoteCandidateTimestampV1,
) -> Result<ReviewNoteCandidateV1, ReviewNoteCandidateTransitionErrorV1> {
    validate_review_note_candidate_v1(current).map_err(invalid_record)?;
    if expected_review_revision != current.review_revision {
        return Err(ReviewNoteCandidateTransitionErrorV1::RevisionConflict);
    }
    if current.state != ReviewNoteCandidateStateV1::Pending {
        return Err(ReviewNoteCandidateTransitionErrorV1::TerminalDecision);
    }
    if !nonzero(&owner_device_id) {
        return Err(ReviewNoteCandidateTransitionErrorV1::InvalidActor);
    }
    if !valid_timestamp(decided_at) || decided_at.unix_seconds < current.updated_at.unix_seconds {
        return Err(ReviewNoteCandidateTransitionErrorV1::InvalidTimestamp);
    }
    let mut next = current.clone();
    next.review_revision = next
        .review_revision
        .checked_add(1)
        .ok_or(ReviewNoteCandidateTransitionErrorV1::RevisionOverflow)?;
    next.decided_by_owner_device_id = Some(owner_device_id);
    next.decided_at = Some(decided_at);
    next.updated_at = decided_at;
    match decision {
        ReviewNoteCandidateDecisionV1::Approve => {
            next.state = ReviewNoteCandidateStateV1::Approved;
            next.promotion_status = ReviewNoteCandidatePromotionStatusV1::Pending;
        }
        ReviewNoteCandidateDecisionV1::Reject => {
            next.state = ReviewNoteCandidateStateV1::Rejected;
            next.promotion_status = ReviewNoteCandidatePromotionStatusV1::NotRequested;
        }
    }
    validate_review_note_candidate_v1(&next).map_err(invalid_record)?;
    Ok(next)
}

pub fn record_review_note_candidate_promotion_v1(
    current: &ReviewNoteCandidateV1,
    expected_review_revision: u64,
    result: ReviewNoteCandidatePromotionResultV1,
    recorded_at: ReviewNoteCandidateTimestampV1,
) -> Result<ReviewNoteCandidateV1, ReviewNoteCandidateTransitionErrorV1> {
    validate_review_note_candidate_v1(current).map_err(invalid_record)?;
    if expected_review_revision != current.review_revision {
        return Err(ReviewNoteCandidateTransitionErrorV1::RevisionConflict);
    }
    if current.state != ReviewNoteCandidateStateV1::Approved
        || current.promotion_status != ReviewNoteCandidatePromotionStatusV1::Pending
    {
        return Err(ReviewNoteCandidateTransitionErrorV1::PromotionNotPending);
    }
    if !valid_timestamp(recorded_at) || recorded_at.unix_seconds < current.updated_at.unix_seconds {
        return Err(ReviewNoteCandidateTransitionErrorV1::InvalidTimestamp);
    }
    let mut next = current.clone();
    next.review_revision = next
        .review_revision
        .checked_add(1)
        .ok_or(ReviewNoteCandidateTransitionErrorV1::RevisionOverflow)?;
    next.updated_at = recorded_at;
    match result {
        ReviewNoteCandidatePromotionResultV1::Succeeded { note_id } => {
            if !nonzero(&note_id) {
                return Err(ReviewNoteCandidateTransitionErrorV1::InvalidRecord);
            }
            next.promotion_status = ReviewNoteCandidatePromotionStatusV1::Succeeded;
            next.promoted_note_id = Some(note_id);
        }
        ReviewNoteCandidatePromotionResultV1::Failed => {
            next.promotion_status = ReviewNoteCandidatePromotionStatusV1::Failed;
        }
    }
    validate_review_note_candidate_v1(&next).map_err(invalid_record)?;
    Ok(next)
}

fn invalid_record(_: ReviewNoteCandidateValidationErrorV1) -> ReviewNoteCandidateTransitionErrorV1 {
    ReviewNoteCandidateTransitionErrorV1::InvalidRecord
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(seconds: i64) -> ReviewNoteCandidateTimestampV1 {
        ReviewNoteCandidateTimestampV1 {
            unix_seconds: seconds,
            nanos: 7,
        }
    }

    fn pending() -> ReviewNoteCandidateV1 {
        create_review_note_candidate_v1(ReviewNoteCandidateDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            source_evidence_id: [3; 16],
            source_evidence_revision: 4,
            title: "Contract approved".to_owned(),
            excerpt: "Invoice amount confirmed".to_owned(),
            topic_hints: vec![
                crate::ReviewNoteTopicHintV1::Financial,
                crate::ReviewNoteTopicHintV1::Legal,
            ],
            source_basis: crate::ReviewNoteSourceBasisV1::Combined,
            confidence_basis_points: 8_300,
            submitted_at: timestamp(1_800_000_000),
        })
        .expect("pending review")
    }

    #[test]
    fn submission_creates_deterministic_pending_review() {
        let first = pending();
        let second = pending();
        assert_eq!(first, second);
        assert_eq!(first.review_revision, 1);
        assert_eq!(first.state, ReviewNoteCandidateStateV1::Pending);
        assert_eq!(
            first.promotion_status,
            ReviewNoteCandidatePromotionStatusV1::NotRequested
        );
    }

    #[test]
    fn approval_is_terminal_and_starts_separate_promotion() {
        let approved = decide_review_note_candidate_v1(
            &pending(),
            1,
            ReviewNoteCandidateDecisionV1::Approve,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("approve");
        assert_eq!(approved.state, ReviewNoteCandidateStateV1::Approved);
        assert_eq!(
            approved.promotion_status,
            ReviewNoteCandidatePromotionStatusV1::Pending
        );
        assert_eq!(
            decide_review_note_candidate_v1(
                &approved,
                2,
                ReviewNoteCandidateDecisionV1::Reject,
                [4; 16],
                timestamp(1_800_000_002),
            ),
            Err(ReviewNoteCandidateTransitionErrorV1::TerminalDecision)
        );
    }

    #[test]
    fn rejection_never_requests_promotion() {
        let rejected = decide_review_note_candidate_v1(
            &pending(),
            1,
            ReviewNoteCandidateDecisionV1::Reject,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("reject");
        assert_eq!(rejected.state, ReviewNoteCandidateStateV1::Rejected);
        assert_eq!(
            rejected.promotion_status,
            ReviewNoteCandidatePromotionStatusV1::NotRequested
        );
        assert_eq!(
            record_review_note_candidate_promotion_v1(
                &rejected,
                2,
                ReviewNoteCandidatePromotionResultV1::Failed,
                timestamp(1_800_000_002),
            ),
            Err(ReviewNoteCandidateTransitionErrorV1::PromotionNotPending)
        );
    }

    #[test]
    fn stale_revision_and_missing_human_actor_are_rejected() {
        assert_eq!(
            decide_review_note_candidate_v1(
                &pending(),
                2,
                ReviewNoteCandidateDecisionV1::Approve,
                [4; 16],
                timestamp(1_800_000_001),
            ),
            Err(ReviewNoteCandidateTransitionErrorV1::RevisionConflict)
        );
        assert_eq!(
            decide_review_note_candidate_v1(
                &pending(),
                1,
                ReviewNoteCandidateDecisionV1::Approve,
                [0; 16],
                timestamp(1_800_000_001),
            ),
            Err(ReviewNoteCandidateTransitionErrorV1::InvalidActor)
        );
    }

    #[test]
    fn terminal_note_result_is_distinct_from_approval() {
        let approved = decide_review_note_candidate_v1(
            &pending(),
            1,
            ReviewNoteCandidateDecisionV1::Approve,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("approve");
        assert_eq!(approved.promoted_note_id, None);
        let succeeded = record_review_note_candidate_promotion_v1(
            &approved,
            2,
            ReviewNoteCandidatePromotionResultV1::Succeeded { note_id: [5; 16] },
            timestamp(1_800_000_002),
        )
        .expect("promotion result");
        assert_eq!(
            succeeded.promotion_status,
            ReviewNoteCandidatePromotionStatusV1::Succeeded
        );
        assert_eq!(succeeded.promoted_note_id, Some([5; 16]));
    }
}
