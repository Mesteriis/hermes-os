#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskSourceBasisV1 {
    Subject,
    Body,
    Combined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskSignalKindV1 {
    ExplicitAction,
    DirectRequest,
    FollowUp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTaskCandidateV1 {
    pub candidate_id: [u8; 16],
    pub candidate_digest: [u8; 32],
    pub title: String,
    pub due_text_hint: Option<String>,
    pub assignee_label_hint: Option<String>,
    pub source_basis: CommunicationTaskSourceBasisV1,
    pub signal_kind: CommunicationTaskSignalKindV1,
    pub confidence_basis_points: u32,
    pub source_evidence_id: [u8; 16],
    pub source_evidence_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationTaskSourceContentV1<'a> {
    pub subject_utf8: &'a [u8],
    pub body_utf8: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTaskCandidateDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub expected_source_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateStateV1 {
    Accepted,
    PreparingSource,
    Extracting,
    Ready,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateCompletenessV1 {
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateRejectionCodeV1 {
    InvalidRequest,
    SourceRejected,
    ExtractionRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTaskCandidateStatusV1 {
    pub state: CommunicationTaskCandidateStateV1,
    pub state_revision: u64,
    pub source_evidence_id: Option<[u8; 16]>,
    pub source_evidence_revision: Option<u64>,
    pub source_sha256: Option<[u8; 32]>,
    pub candidates: Option<Vec<CommunicationTaskCandidateV1>>,
    pub completeness: Option<CommunicationTaskCandidateCompletenessV1>,
    pub rejection: Option<CommunicationTaskCandidateRejectionCodeV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidSourceMessageId,
    InvalidSourceRevision,
}

pub fn validate_communication_task_candidate_draft_v1(
    draft: &CommunicationTaskCandidateDraftV1,
) -> Result<(), CommunicationTaskCandidateValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(CommunicationTaskCandidateValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(CommunicationTaskCandidateValidationErrorV1::InvalidOperationId);
    }
    if zero(&draft.source_message_id) {
        return Err(CommunicationTaskCandidateValidationErrorV1::InvalidSourceMessageId);
    }
    if draft.expected_source_revision == 0 {
        return Err(CommunicationTaskCandidateValidationErrorV1::InvalidSourceRevision);
    }
    Ok(())
}

pub(crate) fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_rejects_zero_identity_and_revision() {
        let valid = CommunicationTaskCandidateDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 1,
        };
        assert_eq!(
            validate_communication_task_candidate_draft_v1(&valid),
            Ok(())
        );

        let mut invalid = valid.clone();
        invalid.source_message_id = [0; 16];
        assert_eq!(
            validate_communication_task_candidate_draft_v1(&invalid),
            Err(CommunicationTaskCandidateValidationErrorV1::InvalidSourceMessageId)
        );
    }
}
