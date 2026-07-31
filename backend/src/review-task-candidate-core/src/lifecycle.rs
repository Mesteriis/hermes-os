use crate::model::{nonzero, valid_timestamp, validate_draft};
use crate::{
    ReviewTaskCandidateDecisionV1, ReviewTaskCandidateDraftV1,
    ReviewTaskCandidatePromotionResultV1, ReviewTaskCandidatePromotionStatusV1,
    ReviewTaskCandidateStateV1, ReviewTaskCandidateTimestampV1, ReviewTaskCandidateV1,
    ReviewTaskCandidateValidationErrorV1, STABLE_ID_BYTES_V1, derive_review_task_candidate_id_v1,
    validate_review_task_candidate_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTaskCandidateTransitionErrorV1 {
    InvalidRecord,
    InvalidActor,
    InvalidTimestamp,
    RevisionConflict,
    TerminalDecision,
    PromotionNotPending,
    RevisionOverflow,
}

pub fn create_review_task_candidate_v1(
    draft: ReviewTaskCandidateDraftV1,
) -> Result<ReviewTaskCandidateV1, ReviewTaskCandidateTransitionErrorV1> {
    validate_draft(&draft).map_err(invalid_record)?;
    let review_id = derive_review_task_candidate_id_v1(
        &draft.logical_owner_id,
        &draft.candidate_id,
        &draft.candidate_digest,
    )
    .map_err(invalid_record)?;
    let review = ReviewTaskCandidateV1 {
        review_id,
        logical_owner_id: draft.logical_owner_id,
        candidate_id: draft.candidate_id,
        candidate_digest: draft.candidate_digest,
        source_evidence_id: draft.source_evidence_id,
        source_evidence_revision: draft.source_evidence_revision,
        title: draft.title,
        due_text_hint: draft.due_text_hint,
        assignee_label_hint: draft.assignee_label_hint,
        state: ReviewTaskCandidateStateV1::Pending,
        promotion_status: ReviewTaskCandidatePromotionStatusV1::NotRequested,
        review_revision: 1,
        decided_by_owner_device_id: None,
        decided_at: None,
        promoted_task_id: None,
        updated_at: draft.submitted_at,
    };
    validate_review_task_candidate_v1(&review).map_err(invalid_record)?;
    Ok(review)
}

pub fn decide_review_task_candidate_v1(
    current: &ReviewTaskCandidateV1,
    expected_review_revision: u64,
    decision: ReviewTaskCandidateDecisionV1,
    owner_device_id: [u8; STABLE_ID_BYTES_V1],
    decided_at: ReviewTaskCandidateTimestampV1,
) -> Result<ReviewTaskCandidateV1, ReviewTaskCandidateTransitionErrorV1> {
    validate_review_task_candidate_v1(current).map_err(invalid_record)?;
    if expected_review_revision != current.review_revision {
        return Err(ReviewTaskCandidateTransitionErrorV1::RevisionConflict);
    }
    if current.state != ReviewTaskCandidateStateV1::Pending {
        return Err(ReviewTaskCandidateTransitionErrorV1::TerminalDecision);
    }
    if !nonzero(&owner_device_id) {
        return Err(ReviewTaskCandidateTransitionErrorV1::InvalidActor);
    }
    if !valid_timestamp(decided_at) || decided_at.unix_seconds < current.updated_at.unix_seconds {
        return Err(ReviewTaskCandidateTransitionErrorV1::InvalidTimestamp);
    }
    let mut next = current.clone();
    next.review_revision = next
        .review_revision
        .checked_add(1)
        .ok_or(ReviewTaskCandidateTransitionErrorV1::RevisionOverflow)?;
    next.decided_by_owner_device_id = Some(owner_device_id);
    next.decided_at = Some(decided_at);
    next.updated_at = decided_at;
    match decision {
        ReviewTaskCandidateDecisionV1::Approve => {
            next.state = ReviewTaskCandidateStateV1::Approved;
            next.promotion_status = ReviewTaskCandidatePromotionStatusV1::Pending;
        }
        ReviewTaskCandidateDecisionV1::Reject => {
            next.state = ReviewTaskCandidateStateV1::Rejected;
            next.promotion_status = ReviewTaskCandidatePromotionStatusV1::NotRequested;
        }
    }
    validate_review_task_candidate_v1(&next).map_err(invalid_record)?;
    Ok(next)
}

pub fn record_review_task_candidate_promotion_v1(
    current: &ReviewTaskCandidateV1,
    expected_review_revision: u64,
    result: ReviewTaskCandidatePromotionResultV1,
    recorded_at: ReviewTaskCandidateTimestampV1,
) -> Result<ReviewTaskCandidateV1, ReviewTaskCandidateTransitionErrorV1> {
    validate_review_task_candidate_v1(current).map_err(invalid_record)?;
    if expected_review_revision != current.review_revision {
        return Err(ReviewTaskCandidateTransitionErrorV1::RevisionConflict);
    }
    if current.state != ReviewTaskCandidateStateV1::Approved
        || current.promotion_status != ReviewTaskCandidatePromotionStatusV1::Pending
    {
        return Err(ReviewTaskCandidateTransitionErrorV1::PromotionNotPending);
    }
    if !valid_timestamp(recorded_at) || recorded_at.unix_seconds < current.updated_at.unix_seconds {
        return Err(ReviewTaskCandidateTransitionErrorV1::InvalidTimestamp);
    }
    let mut next = current.clone();
    next.review_revision = next
        .review_revision
        .checked_add(1)
        .ok_or(ReviewTaskCandidateTransitionErrorV1::RevisionOverflow)?;
    next.updated_at = recorded_at;
    match result {
        ReviewTaskCandidatePromotionResultV1::Succeeded { task_id } => {
            if !nonzero(&task_id) {
                return Err(ReviewTaskCandidateTransitionErrorV1::InvalidRecord);
            }
            next.promotion_status = ReviewTaskCandidatePromotionStatusV1::Succeeded;
            next.promoted_task_id = Some(task_id);
        }
        ReviewTaskCandidatePromotionResultV1::Failed => {
            next.promotion_status = ReviewTaskCandidatePromotionStatusV1::Failed;
        }
    }
    validate_review_task_candidate_v1(&next).map_err(invalid_record)?;
    Ok(next)
}

fn invalid_record(_: ReviewTaskCandidateValidationErrorV1) -> ReviewTaskCandidateTransitionErrorV1 {
    ReviewTaskCandidateTransitionErrorV1::InvalidRecord
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(seconds: i64) -> ReviewTaskCandidateTimestampV1 {
        ReviewTaskCandidateTimestampV1 {
            unix_seconds: seconds,
            nanos: 7,
        }
    }

    fn pending() -> ReviewTaskCandidateV1 {
        create_review_task_candidate_v1(ReviewTaskCandidateDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            source_evidence_id: [3; 16],
            source_evidence_revision: 4,
            title: "Подготовить ответ".to_owned(),
            due_text_hint: Some("до пятницы".to_owned()),
            assignee_label_hint: None,
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
        assert_eq!(first.state, ReviewTaskCandidateStateV1::Pending);
        assert_eq!(
            first.promotion_status,
            ReviewTaskCandidatePromotionStatusV1::NotRequested
        );
    }

    #[test]
    fn approval_is_terminal_and_starts_separate_promotion() {
        let approved = decide_review_task_candidate_v1(
            &pending(),
            1,
            ReviewTaskCandidateDecisionV1::Approve,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("approve");
        assert_eq!(approved.state, ReviewTaskCandidateStateV1::Approved);
        assert_eq!(
            approved.promotion_status,
            ReviewTaskCandidatePromotionStatusV1::Pending
        );
        assert_eq!(
            decide_review_task_candidate_v1(
                &approved,
                2,
                ReviewTaskCandidateDecisionV1::Reject,
                [4; 16],
                timestamp(1_800_000_002),
            ),
            Err(ReviewTaskCandidateTransitionErrorV1::TerminalDecision)
        );
    }

    #[test]
    fn rejection_never_requests_promotion() {
        let rejected = decide_review_task_candidate_v1(
            &pending(),
            1,
            ReviewTaskCandidateDecisionV1::Reject,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("reject");
        assert_eq!(rejected.state, ReviewTaskCandidateStateV1::Rejected);
        assert_eq!(
            rejected.promotion_status,
            ReviewTaskCandidatePromotionStatusV1::NotRequested
        );
        assert_eq!(
            record_review_task_candidate_promotion_v1(
                &rejected,
                2,
                ReviewTaskCandidatePromotionResultV1::Failed,
                timestamp(1_800_000_002),
            ),
            Err(ReviewTaskCandidateTransitionErrorV1::PromotionNotPending)
        );
    }

    #[test]
    fn stale_revision_and_missing_human_actor_are_rejected() {
        assert_eq!(
            decide_review_task_candidate_v1(
                &pending(),
                2,
                ReviewTaskCandidateDecisionV1::Approve,
                [4; 16],
                timestamp(1_800_000_001),
            ),
            Err(ReviewTaskCandidateTransitionErrorV1::RevisionConflict)
        );
        assert_eq!(
            decide_review_task_candidate_v1(
                &pending(),
                1,
                ReviewTaskCandidateDecisionV1::Approve,
                [0; 16],
                timestamp(1_800_000_001),
            ),
            Err(ReviewTaskCandidateTransitionErrorV1::InvalidActor)
        );
    }

    #[test]
    fn terminal_task_result_is_distinct_from_approval() {
        let approved = decide_review_task_candidate_v1(
            &pending(),
            1,
            ReviewTaskCandidateDecisionV1::Approve,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("approve");
        assert_eq!(approved.promoted_task_id, None);
        let succeeded = record_review_task_candidate_promotion_v1(
            &approved,
            2,
            ReviewTaskCandidatePromotionResultV1::Succeeded { task_id: [5; 16] },
            timestamp(1_800_000_002),
        )
        .expect("promotion result");
        assert_eq!(
            succeeded.promotion_status,
            ReviewTaskCandidatePromotionStatusV1::Succeeded
        );
        assert_eq!(succeeded.promoted_task_id, Some([5; 16]));
    }
}
