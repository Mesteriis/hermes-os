const MAX_PREVIEW_SOURCE_BYTES_V1: u64 = 100 * 1024 * 1024;
const MAX_CUSTODY_SOURCE_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewRequestFactV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub logical_owner_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewScanCandidateFactV1 {
    pub attachment_anchor_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub candidate_envelope_sha256: [u8; 32],
    pub source_reference_id: [u8; 16],
    pub declared_size: u64,
    pub source_receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewSafetyStateV1 {
    DescriptorOnly,
    BlobPending,
    BlobAdmitted,
    SafeForDelivery,
    Quarantined,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewSafetyFactV1 {
    pub attachment_anchor_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub safety_evidence_id: [u8; 16],
    pub expected_state: AttachmentPreviewSafetyStateV1,
    pub next_state: AttachmentPreviewSafetyStateV1,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewCustodyDelegationIntentV1 {
    pub run_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub logical_owner_id: String,
    pub candidate_message_id: [u8; 16],
    pub candidate_envelope_sha256: [u8; 32],
    pub safety_message_id: [u8; 16],
    pub safety_evidence_id: [u8; 16],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
        validate_request(&fact)?;
        replace_exact(&mut self.request, fact)
    }

    pub fn observe_candidate(
        &mut self,
        fact: AttachmentPreviewScanCandidateFactV1,
    ) -> Result<(), AttachmentPreviewJoinErrorV1> {
        validate_candidate(&fact)?;
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
        let Some(request) = self.request.as_ref() else {
            return Ok(None);
        };
        if let Some(safety) = self.safety {
            if request.attachment_anchor_id != safety.attachment_anchor_id {
                return Err(AttachmentPreviewJoinErrorV1::Collision);
            }
            if safety.expected_state != AttachmentPreviewSafetyStateV1::BlobAdmitted
                || safety.next_state != AttachmentPreviewSafetyStateV1::SafeForDelivery
            {
                return Err(AttachmentPreviewJoinErrorV1::NotSafe);
            }
        }
        let (Some(candidate), Some(safety)) = (self.candidate.as_ref(), self.safety) else {
            return Ok(None);
        };
        if request.attachment_anchor_id != candidate.attachment_anchor_id {
            return Err(AttachmentPreviewJoinErrorV1::Collision);
        }
        Ok(Some(AttachmentPreviewCustodyDelegationIntentV1 {
            run_id: request.run_id,
            attachment_anchor_id: request.attachment_anchor_id,
            logical_owner_id: request.logical_owner_id.clone(),
            candidate_message_id: candidate.candidate_message_id,
            candidate_envelope_sha256: candidate.candidate_envelope_sha256,
            safety_message_id: safety.safety_message_id,
            safety_evidence_id: safety.safety_evidence_id,
        }))
    }
}

fn replace_exact<T: Clone + Eq>(
    slot: &mut Option<T>,
    value: T,
) -> Result<(), AttachmentPreviewJoinErrorV1> {
    match slot {
        Some(current) if *current != value => Err(AttachmentPreviewJoinErrorV1::Collision),
        Some(_) => Ok(()),
        None => {
            *slot = Some(value);
            Ok(())
        }
    }
}

fn validate_request(
    fact: &AttachmentPreviewRequestFactV1,
) -> Result<(), AttachmentPreviewJoinErrorV1> {
    valid_id(fact.run_id)?;
    valid_id(fact.operation_id)?;
    valid_id(fact.attachment_anchor_id)?;
    if fact.logical_owner_id.is_empty()
        || fact.logical_owner_id.len() > 128
        || !fact.logical_owner_id.is_ascii()
    {
        Err(AttachmentPreviewJoinErrorV1::InvalidFact)
    } else {
        Ok(())
    }
}

fn validate_candidate(
    fact: &AttachmentPreviewScanCandidateFactV1,
) -> Result<(), AttachmentPreviewJoinErrorV1> {
    valid_id(fact.attachment_anchor_id)?;
    valid_id(fact.candidate_message_id)?;
    valid_sha256(fact.candidate_envelope_sha256)?;
    valid_id(fact.source_reference_id)?;
    valid_sha256(fact.source_receipt_sha256)?;
    if !(1..=MAX_PREVIEW_SOURCE_BYTES_V1).contains(&fact.declared_size)
        || !(1..=MAX_CUSTODY_SOURCE_PROOF_BYTES_V1)
            .contains(&fact.custody_transfer_source_proof.len())
        || fact.observed_at_unix_seconds <= 0
    {
        return Err(AttachmentPreviewJoinErrorV1::InvalidFact);
    }
    Ok(())
}

fn validate_safety(
    fact: AttachmentPreviewSafetyFactV1,
) -> Result<(), AttachmentPreviewJoinErrorV1> {
    valid_id(fact.attachment_anchor_id)?;
    valid_id(fact.safety_message_id)?;
    valid_id(fact.safety_evidence_id)?;
    if fact.observed_at_unix_seconds <= 0
        || !matches!(
            fact.next_state,
            AttachmentPreviewSafetyStateV1::SafeForDelivery
                | AttachmentPreviewSafetyStateV1::Quarantined
                | AttachmentPreviewSafetyStateV1::Rejected
        )
        || (fact.next_state == AttachmentPreviewSafetyStateV1::SafeForDelivery
            && fact.expected_state != AttachmentPreviewSafetyStateV1::BlobAdmitted)
        || fact.expected_state == fact.next_state
    {
        return Err(AttachmentPreviewJoinErrorV1::InvalidFact);
    }
    Ok(())
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
    fn join_is_order_independent_and_intent_carries_no_blob_authority() {
        let request = request();
        let candidate = candidate();
        let safety = safety(AttachmentPreviewSafetyStateV1::SafeForDelivery);
        let mut first = AttachmentPreviewEvidenceJoinV1::default();
        first.observe_request(request.clone()).unwrap();
        first.observe_candidate(candidate.clone()).unwrap();
        first.observe_safety(safety).unwrap();
        let mut second = AttachmentPreviewEvidenceJoinV1::default();
        second.observe_safety(safety).unwrap();
        second.observe_candidate(candidate).unwrap();
        second.observe_request(request).unwrap();
        assert_eq!(first.delegation_intent(), second.delegation_intent());
    }

    #[test]
    fn candidate_collision_and_non_safe_transition_fail_closed() {
        let mut safety_only = AttachmentPreviewEvidenceJoinV1::default();
        safety_only.observe_request(request()).unwrap();
        safety_only
            .observe_safety(safety(AttachmentPreviewSafetyStateV1::Quarantined))
            .unwrap();
        assert_eq!(
            safety_only.delegation_intent(),
            Err(AttachmentPreviewJoinErrorV1::NotSafe)
        );
        let mut join = AttachmentPreviewEvidenceJoinV1::default();
        join.observe_request(request()).unwrap();
        join.observe_candidate(candidate()).unwrap();
        join.observe_safety(safety(AttachmentPreviewSafetyStateV1::Quarantined))
            .unwrap();
        assert_eq!(
            join.delegation_intent(),
            Err(AttachmentPreviewJoinErrorV1::NotSafe)
        );
        let mut collision = candidate();
        collision.declared_size += 1;
        assert_eq!(
            join.observe_candidate(collision),
            Err(AttachmentPreviewJoinErrorV1::Collision)
        );
    }

    #[test]
    fn oversized_source_and_proof_are_invalid() {
        let mut oversized = candidate();
        oversized.declared_size = MAX_PREVIEW_SOURCE_BYTES_V1 + 1;
        assert_eq!(
            AttachmentPreviewEvidenceJoinV1::default().observe_candidate(oversized),
            Err(AttachmentPreviewJoinErrorV1::InvalidFact)
        );
        let mut oversized_proof = candidate();
        oversized_proof.custody_transfer_source_proof =
            vec![7; MAX_CUSTODY_SOURCE_PROOF_BYTES_V1 + 1];
        assert_eq!(
            AttachmentPreviewEvidenceJoinV1::default().observe_candidate(oversized_proof),
            Err(AttachmentPreviewJoinErrorV1::InvalidFact)
        );
    }

    fn request() -> AttachmentPreviewRequestFactV1 {
        AttachmentPreviewRequestFactV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            attachment_anchor_id: [3; 16],
            logical_owner_id: "owner-a".to_owned(),
        }
    }

    fn candidate() -> AttachmentPreviewScanCandidateFactV1 {
        AttachmentPreviewScanCandidateFactV1 {
            attachment_anchor_id: [3; 16],
            candidate_message_id: [5; 16],
            candidate_envelope_sha256: [6; 32],
            source_reference_id: [7; 16],
            declared_size: 42,
            source_receipt_sha256: [8; 32],
            custody_transfer_source_proof: vec![9; 64],
            observed_at_unix_seconds: 1_800_000_000,
        }
    }

    fn safety(next_state: AttachmentPreviewSafetyStateV1) -> AttachmentPreviewSafetyFactV1 {
        AttachmentPreviewSafetyFactV1 {
            attachment_anchor_id: [3; 16],
            safety_message_id: [10; 16],
            safety_evidence_id: [11; 16],
            expected_state: AttachmentPreviewSafetyStateV1::BlobAdmitted,
            next_state,
            observed_at_unix_seconds: 1_800_000_001,
        }
    }
}
