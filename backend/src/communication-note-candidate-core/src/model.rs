#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteSourceBasisV1 {
    Subject,
    Body,
    Combined,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CommunicationNoteTopicHintV1 {
    Financial,
    Legal,
    DecisionStatement,
    DeadlineStatement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationNoteCandidateV1 {
    pub candidate_id: [u8; 16],
    pub candidate_digest: [u8; 32],
    pub title: String,
    pub excerpt: String,
    pub topic_hints: Vec<CommunicationNoteTopicHintV1>,
    pub source_basis: CommunicationNoteSourceBasisV1,
    pub confidence_basis_points: u32,
    pub source_evidence_id: [u8; 16],
    pub source_evidence_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationNoteSourceContentV1<'a> {
    pub subject_utf8: &'a [u8],
    pub body_utf8: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationNoteCandidateDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub expected_source_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateStateV1 {
    Accepted,
    PreparingSource,
    Extracting,
    Ready,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateCompletenessV1 {
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateRejectionCodeV1 {
    InvalidRequest,
    SourceRejected,
    ExtractionRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationNoteCandidateStatusV1 {
    pub state: CommunicationNoteCandidateStateV1,
    pub state_revision: u64,
    pub source_evidence_id: Option<[u8; 16]>,
    pub source_evidence_revision: Option<u64>,
    pub source_sha256: Option<[u8; 32]>,
    pub candidates: Option<Vec<CommunicationNoteCandidateV1>>,
    pub completeness: Option<CommunicationNoteCandidateCompletenessV1>,
    pub rejection: Option<CommunicationNoteCandidateRejectionCodeV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidSourceMessageId,
    InvalidSourceRevision,
}

pub fn validate_communication_note_candidate_draft_v1(
    draft: &CommunicationNoteCandidateDraftV1,
) -> Result<(), CommunicationNoteCandidateValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(CommunicationNoteCandidateValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(CommunicationNoteCandidateValidationErrorV1::InvalidOperationId);
    }
    if zero(&draft.source_message_id) {
        return Err(CommunicationNoteCandidateValidationErrorV1::InvalidSourceMessageId);
    }
    if draft.expected_source_revision == 0 {
        return Err(CommunicationNoteCandidateValidationErrorV1::InvalidSourceRevision);
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
        let valid = CommunicationNoteCandidateDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 1,
        };
        assert_eq!(
            validate_communication_note_candidate_draft_v1(&valid),
            Ok(())
        );

        let mut invalid = valid.clone();
        invalid.source_message_id = [0; 16];
        assert_eq!(
            validate_communication_note_candidate_draft_v1(&invalid),
            Err(CommunicationNoteCandidateValidationErrorV1::InvalidSourceMessageId)
        );
    }
}
