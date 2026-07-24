//! Typed conversion from owner query values to the public Protobuf contract.

use crate::{
    AttachmentDispositionV1, AttachmentSafetyStateV1, CommunicationAccountSummaryV1,
    CommunicationAttachmentAnchorSummaryV1, CommunicationConversationSummaryV1,
    CommunicationDirectionV1, CommunicationMessageLifecycleStateV1,
    CommunicationMessageReferenceKindV1, CommunicationMessageReferenceSummaryV1,
    CommunicationMessageSummaryV1, CommunicationObservedParticipantSummaryV1,
    CommunicationProviderProvenanceV1, query_wire,
};

impl From<&crate::CommunicationSummary> for query_wire::EvidenceSummaryV1 {
    fn from(value: &crate::CommunicationSummary) -> Self {
        Self {
            evidence_id: value.evidence_id.bytes().to_vec(),
            provider: provider_value(value.provider),
            direction: direction_value(value.direction),
            kind: evidence_kind_value(value.kind),
            body_state: body_state_value(value.body),
            body_admission_failure: body_admission_failure_value(value.body_admission_failure),
            observed_at_unix_seconds: value.observed_at_unix_seconds,
        }
    }
}

impl From<&CommunicationAccountSummaryV1> for query_wire::AccountSummaryV1 {
    fn from(value: &CommunicationAccountSummaryV1) -> Self {
        Self {
            account_id: value.account_id.bytes().to_vec(),
            account_cursor_sha256: value.account_cursor.bytes().to_vec(),
            provider: provider_value(value.provider),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
        }
    }
}

impl From<&CommunicationConversationSummaryV1> for query_wire::ConversationSummaryV1 {
    fn from(value: &CommunicationConversationSummaryV1) -> Self {
        Self {
            conversation_id: value.conversation_id.bytes().to_vec(),
            account_cursor_sha256: value.account_cursor.bytes().to_vec(),
            conversation_cursor_sha256: value.conversation_cursor.bytes().to_vec(),
            provider: provider_value(value.provider),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
        }
    }
}

impl From<&CommunicationMessageSummaryV1> for query_wire::MessageSummaryV1 {
    fn from(value: &CommunicationMessageSummaryV1) -> Self {
        Self {
            message_id: value.message_id.bytes().to_vec(),
            conversation_id: value.conversation_id.bytes().to_vec(),
            source_cursor_sha256: value.source_cursor.bytes().to_vec(),
            body_state: body_state_value(value.body),
            direction: direction_value(value.direction),
            lifecycle_state: lifecycle_state_value(value.lifecycle_state),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
        }
    }
}

impl From<&CommunicationObservedParticipantSummaryV1> for query_wire::ObservedParticipantSummaryV1 {
    fn from(value: &CommunicationObservedParticipantSummaryV1) -> Self {
        Self {
            participant_id: value.participant_id.bytes().to_vec(),
            conversation_id: value.conversation_id.bytes().to_vec(),
            participant_cursor_sha256: value.participant_cursor.bytes().to_vec(),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
        }
    }
}

impl From<&CommunicationAttachmentAnchorSummaryV1> for query_wire::AttachmentAnchorSummaryV1 {
    fn from(value: &CommunicationAttachmentAnchorSummaryV1) -> Self {
        Self {
            attachment_anchor_id: value.attachment_anchor_id.bytes().to_vec(),
            message_id: value.message_id.bytes().to_vec(),
            media_cursor_sha256: value.media_cursor.bytes().to_vec(),
            state: attachment_state_value(value.state),
            first_observed_at_unix_seconds: value.first_observed_at_unix_seconds,
            last_observed_at_unix_seconds: value.last_observed_at_unix_seconds,
            last_evidence_id: value.last_evidence_id.bytes().to_vec(),
            has_descriptor: value.descriptor.is_some(),
            filename: value
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.filename())
                .unwrap_or_default()
                .to_owned(),
            has_filename: value
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.filename())
                .is_some(),
            media_type: value
                .descriptor
                .as_ref()
                .map(|descriptor| descriptor.media_type())
                .unwrap_or_default()
                .to_owned(),
            declared_bytes: value
                .descriptor
                .as_ref()
                .map_or(0, |descriptor| descriptor.declared_bytes()),
            sha256: value
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.sha256())
                .map_or_else(Vec::new, |value| value.to_vec()),
            disposition: value.descriptor.as_ref().map_or(0, |descriptor| {
                match descriptor.disposition() {
                    AttachmentDispositionV1::Attachment => 1,
                    AttachmentDispositionV1::Inline => 2,
                    AttachmentDispositionV1::Unknown => 3,
                }
            }),
        }
    }
}

impl From<&CommunicationMessageReferenceSummaryV1> for query_wire::MessageReferenceSummaryV1 {
    fn from(value: &CommunicationMessageReferenceSummaryV1) -> Self {
        Self {
            source_message_id: value.source_message_id.bytes().to_vec(),
            kind: reference_kind_value(value.kind),
            target_source_cursor_sha256: value.target_source_cursor.bytes().to_vec(),
            target_message_id: value
                .target_message_id
                .map_or_else(Vec::new, |id| id.bytes().to_vec()),
            observed_at_unix_seconds: value.observed_at_unix_seconds,
            evidence_id: value.evidence_id.bytes().to_vec(),
        }
    }
}

const fn provider_value(value: CommunicationProviderProvenanceV1) -> u32 {
    match value {
        CommunicationProviderProvenanceV1::MailImap => 1,
        CommunicationProviderProvenanceV1::Telegram => 2,
        CommunicationProviderProvenanceV1::WhatsAppWeb => 3,
        CommunicationProviderProvenanceV1::MailSmtp => 4,
        CommunicationProviderProvenanceV1::Zulip => 5,
        CommunicationProviderProvenanceV1::MailGmail => 6,
    }
}

const fn direction_value(value: CommunicationDirectionV1) -> u32 {
    match value {
        CommunicationDirectionV1::Incoming => 1,
        CommunicationDirectionV1::Outgoing => 2,
        CommunicationDirectionV1::Unknown => 3,
    }
}

const fn body_state_value(value: crate::CommunicationBodyStateV1) -> u32 {
    match value {
        crate::CommunicationBodyStateV1::MetadataOnly => 1,
        crate::CommunicationBodyStateV1::PendingBlob => 2,
        crate::CommunicationBodyStateV1::Unavailable => 3,
        crate::CommunicationBodyStateV1::AdmittedBlob => 4,
    }
}

const fn body_admission_failure_value(
    value: Option<crate::CommunicationBodyAdmissionFailureV1>,
) -> u32 {
    match value {
        None => 0,
        Some(crate::CommunicationBodyAdmissionFailureV1::SourceUnavailable) => 1,
        Some(crate::CommunicationBodyAdmissionFailureV1::SizeLimitExceeded) => 2,
        Some(crate::CommunicationBodyAdmissionFailureV1::IntegrityMismatch) => 3,
        Some(crate::CommunicationBodyAdmissionFailureV1::PolicyRejected) => 4,
    }
}

const fn evidence_kind_value(value: crate::CanonicalCommunicationEvidenceKindV1) -> u32 {
    match value {
        crate::CanonicalCommunicationEvidenceKindV1::EmailMessage => 1,
        crate::CanonicalCommunicationEvidenceKindV1::ChatMessage => 2,
        crate::CanonicalCommunicationEvidenceKindV1::MessageEdited => 3,
        crate::CanonicalCommunicationEvidenceKindV1::MessageDeleted => 4,
        crate::CanonicalCommunicationEvidenceKindV1::ReactionChanged => 5,
        crate::CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged => 6,
        crate::CanonicalCommunicationEvidenceKindV1::ConversationStateChanged => 7,
        crate::CanonicalCommunicationEvidenceKindV1::ParticipantChanged => 8,
        crate::CanonicalCommunicationEvidenceKindV1::MediaChanged => 9,
        crate::CanonicalCommunicationEvidenceKindV1::TopicChanged => 10,
        crate::CanonicalCommunicationEvidenceKindV1::TypingChanged => 11,
    }
}

const fn lifecycle_state_value(value: CommunicationMessageLifecycleStateV1) -> u32 {
    match value {
        CommunicationMessageLifecycleStateV1::Active => 1,
        CommunicationMessageLifecycleStateV1::Deleted => 2,
    }
}

const fn reference_kind_value(value: CommunicationMessageReferenceKindV1) -> u32 {
    match value {
        CommunicationMessageReferenceKindV1::Reply => 1,
        CommunicationMessageReferenceKindV1::Forward => 2,
    }
}

const fn attachment_state_value(value: AttachmentSafetyStateV1) -> u32 {
    match value {
        AttachmentSafetyStateV1::DescriptorOnly => 1,
        AttachmentSafetyStateV1::BlobPending => 2,
        AttachmentSafetyStateV1::BlobAdmitted => 3,
        AttachmentSafetyStateV1::Quarantined => 4,
        AttachmentSafetyStateV1::SafeForDelivery => 5,
        AttachmentSafetyStateV1::Rejected => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CanonicalCommunicationEvidenceKindV1, CommunicationBodyAdmissionFailureV1,
        CommunicationBodyBlobReferenceV1, CommunicationBodyStateV1, CommunicationDirectionV1,
        CommunicationObservationIdV1, CommunicationProviderProvenanceV1,
        CommunicationSourceCursorV1, CommunicationSummary,
    };

    #[test]
    fn evidence_summary_exposes_only_canonical_metadata() {
        let summary = CommunicationSummary {
            evidence_id: CommunicationObservationIdV1::new([1; 16]),
            observation_id: CommunicationObservationIdV1::new([2; 16]),
            source_cursor: CommunicationSourceCursorV1::new([3; 32]),
            account_cursor: Some(CommunicationSourceCursorV1::new([4; 32])),
            conversation_cursor: Some(CommunicationSourceCursorV1::new([5; 32])),
            participant_cursor: None,
            media_cursor: None,
            reply_to_source_cursor: None,
            forward_origin_source_cursor: None,
            provider: CommunicationProviderProvenanceV1::MailImap,
            direction: CommunicationDirectionV1::Incoming,
            kind: CanonicalCommunicationEvidenceKindV1::EmailMessage,
            body: CommunicationBodyStateV1::AdmittedBlob,
            body_blob: Some(CommunicationBodyBlobReferenceV1 {
                blob_ref: "private-blob-locator".to_owned(),
                reference_id: [6; 16],
                declared_bytes: 64,
                sha256: [7; 32],
            }),
            body_admission_failure: Some(CommunicationBodyAdmissionFailureV1::PolicyRejected),
            attachment_descriptor: None,
            observed_at_unix_seconds: 8,
        };

        let wire: query_wire::EvidenceSummaryV1 = (&summary).into();

        assert_eq!(wire.evidence_id, vec![1; 16]);
        assert_eq!(wire.provider, 1);
        assert_eq!(wire.direction, 1);
        assert_eq!(wire.kind, 1);
        assert_eq!(wire.body_state, 4);
        assert_eq!(wire.body_admission_failure, 4);
        assert_eq!(wire.observed_at_unix_seconds, 8);
    }
}
