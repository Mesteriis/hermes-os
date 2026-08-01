#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewRequestFactV1 {
    pub run_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub logical_owner_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewScanCandidateFactV1 {
    pub attachment_anchor_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub candidate_envelope_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewSafetyFactV1 {
    pub attachment_anchor_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub safety_evidence_id: [u8; 16],
    pub safe_for_delivery: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewCustodyDelegationIntentV1 {
    pub run_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub logical_owner_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub candidate_envelope_sha256: [u8; 32],
    pub safety_message_id: [u8; 16],
    pub safety_evidence_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AttachmentPreviewEvidenceJoinV1 {
    request: Option<AttachmentPreviewRequestFactV1>,
    candidate: Option<AttachmentPreviewScanCandidateFactV1>,
    safety: Option<AttachmentPreviewSafetyFactV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewJoinErrorV1 {
    InvalidFact,
    Collision,
    NotSafe,
}

impl AttachmentPreviewEvidenceJoinV1 {
    pub fn observe_request(
        &mut self,
        fact: AttachmentPreviewRequestFactV1,
    ) -> Result<(), AttachmentPreviewJoinErrorV1> {
        validate_request(fact)?;
        replace_exact(&mut self.request, fact)
    }

    pub fn observe_candidate(
        &mut self,
        fact: AttachmentPreviewScanCandidateFactV1,
    ) -> Result<(), AttachmentPreviewJoinErrorV1> {
        validate_candidate(fact)?;
        replace_exact(&mut self.candidate, fact)
    }

    pub fn observe_safety(
        &mut self,
        fact: AttachmentPreviewSafetyFactV1,
    ) -> Result<(), AttachmentPreviewJoinErrorV1> {
        validate_safety(fact)?;
        replace_exact(&mut self.safety, fact)
    }

    pub fn delegation_intent(
        &self,
    ) -> Result<Option<AttachmentPreviewCustodyDelegationIntentV1>, AttachmentPreviewJoinErrorV1>
    {
        let (Some(request), Some(candidate), Some(safety)) =
            (self.request, self.candidate, self.safety)
        else {
            return Ok(None);
        };
        if request.attachment_anchor_id != candidate.attachment_anchor_id
            || request.attachment_anchor_id != safety.attachment_anchor_id
        {
            return Err(AttachmentPreviewJoinErrorV1::Collision);
        }
        if !safety.safe_for_delivery {
            return Err(AttachmentPreviewJoinErrorV1::NotSafe);
        }
        Ok(Some(AttachmentPreviewCustodyDelegationIntentV1 {
            run_id: request.run_id,
            attachment_anchor_id: request.attachment_anchor_id,
            logical_owner_id: request.logical_owner_id,
            candidate_message_id: candidate.candidate_message_id,
            candidate_envelope_sha256: candidate.candidate_envelope_sha256,
            safety_message_id: safety.safety_message_id,
            safety_evidence_id: safety.safety_evidence_id,
        }))
    }
}

fn replace_exact<T: Copy + Eq>(
    slot: &mut Option<T>,
    value: T,
) -> Result<(), AttachmentPreviewJoinErrorV1> {
    match *slot {
        Some(current) if current != value => Err(AttachmentPreviewJoinErrorV1::Collision),
        Some(_) => Ok(()),
        None => {
            *slot = Some(value);
            Ok(())
        }
    }
}

fn validate_request(
    fact: AttachmentPreviewRequestFactV1,
) -> Result<(), AttachmentPreviewJoinErrorV1> {
    valid_id(fact.run_id)?;
    valid_id(fact.attachment_anchor_id)?;
    valid_id(fact.logical_owner_id)
}

fn validate_candidate(
    fact: AttachmentPreviewScanCandidateFactV1,
) -> Result<(), AttachmentPreviewJoinErrorV1> {
    valid_id(fact.attachment_anchor_id)?;
    valid_id(fact.candidate_message_id)?;
    valid_sha256(fact.candidate_envelope_sha256)
}

fn validate_safety(
    fact: AttachmentPreviewSafetyFactV1,
) -> Result<(), AttachmentPreviewJoinErrorV1> {
    valid_id(fact.attachment_anchor_id)?;
    valid_id(fact.safety_message_id)?;
    valid_id(fact.safety_evidence_id)
}

fn valid_id(value: [u8; 16]) -> Result<(), AttachmentPreviewJoinErrorV1> {
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(())
        .ok_or(AttachmentPreviewJoinErrorV1::InvalidFact)
}

fn valid_sha256(value: [u8; 32]) -> Result<(), AttachmentPreviewJoinErrorV1> {
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(())
        .ok_or(AttachmentPreviewJoinErrorV1::InvalidFact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_is_order_independent_and_carries_no_blob_authority() {
        let request = AttachmentPreviewRequestFactV1 {
            run_id: [1; 16],
            attachment_anchor_id: [2; 16],
            logical_owner_id: [3; 16],
        };
        let candidate = AttachmentPreviewScanCandidateFactV1 {
            attachment_anchor_id: [2; 16],
            candidate_message_id: [4; 16],
            candidate_envelope_sha256: [5; 32],
        };
        let safety = AttachmentPreviewSafetyFactV1 {
            attachment_anchor_id: [2; 16],
            safety_message_id: [6; 16],
            safety_evidence_id: [7; 16],
            safe_for_delivery: true,
        };
        let mut first = AttachmentPreviewEvidenceJoinV1::default();
        first.observe_request(request).unwrap();
        first.observe_candidate(candidate).unwrap();
        first.observe_safety(safety).unwrap();
        let mut second = AttachmentPreviewEvidenceJoinV1::default();
        second.observe_safety(safety).unwrap();
        second.observe_candidate(candidate).unwrap();
        second.observe_request(request).unwrap();
        assert_eq!(first.delegation_intent(), second.delegation_intent());
    }

    #[test]
    fn collision_and_unsafe_evidence_fail_closed() {
        let mut join = AttachmentPreviewEvidenceJoinV1::default();
        join.observe_request(AttachmentPreviewRequestFactV1 {
            run_id: [1; 16],
            attachment_anchor_id: [2; 16],
            logical_owner_id: [3; 16],
        })
        .unwrap();
        assert_eq!(
            join.observe_request(AttachmentPreviewRequestFactV1 {
                run_id: [1; 16],
                attachment_anchor_id: [9; 16],
                logical_owner_id: [3; 16],
            }),
            Err(AttachmentPreviewJoinErrorV1::Collision)
        );
    }
}
