//! Canonical evidence decisions owned exclusively by Communications.

use hermes_communications_api::{
    CanonicalCommunicationEvidenceKindV1, CanonicalCommunicationProjectionV1,
    CanonicalAccountProjectionV1, CommunicationAccountIdV1,
    CanonicalConversationProjectionV1, CanonicalMessageMutationV1,
    CanonicalMessageProjectionV1, CommunicationConversationIdV1,
    CanonicalObservedParticipantProjectionV1, CommunicationMessageIdV1,
    CanonicalAttachmentAnchorProjectionV1, CommunicationAttachmentAnchorIdV1,
    CanonicalMessageReferenceProjectionV1, CommunicationMessageReferenceKindV1,
    CommunicationObservationIdV1, CommunicationParticipantIdV1, CommunicationSummary,
    CommunicationsClientError, RecordCommunicationEvidenceV1,
    AttachmentSafetyTransitionCommandV1, AttachmentSafetyTransitionDecisionV1,
};
use hermes_communications_api::PACKAGE as API_PACKAGE;
use sha2::{Digest, Sha256};

mod search;
pub use search::{
    COMMUNICATIONS_SEARCH_MAX_DOCUMENT_BYTES_V1, COMMUNICATIONS_SEARCH_MAX_DOCUMENT_TOKENS_V1,
    COMMUNICATIONS_SEARCH_MAX_QUERY_BYTES_V1, COMMUNICATIONS_SEARCH_MAX_QUERY_TOKENS_V1,
    CommunicationsSearchDocumentV1, CommunicationsSearchIndexDecisionV1,
    CommunicationsSearchIndexJobV1, CommunicationsSearchQueryV1, CommunicationsSearchTokenErrorV1,
    decide_search_index_v1, normalize_search_document_tokens_v1, normalize_search_query_v1,
};

pub const PACKAGE: &str = "hermes-communications-domain";
pub fn dependency() -> &'static str { API_PACKAGE }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCommunication { pub evidence_id: CommunicationObservationIdV1, pub summary: CommunicationSummary }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDomainError {
    InvalidObservedTime,
    MissingMessageScope,
    InvalidAttachmentScope,
    InvalidAttachmentSafetyTransition,
}

pub fn accept_command(command: RecordCommunicationEvidenceV1) -> Result<CommunicationSummary, CommunicationsDomainError> {
    validate_body_admission(&command)?;
    if command.attachment_descriptor.is_some()
        && (command.kind != CanonicalCommunicationEvidenceKindV1::MediaChanged || command.media_cursor.is_none())
    {
        return Err(CommunicationsDomainError::InvalidAttachmentScope);
    }
    if matches!(command.kind, CanonicalCommunicationEvidenceKindV1::EmailMessage | CanonicalCommunicationEvidenceKindV1::ChatMessage)
        && (command.account_cursor.is_none() || command.conversation_cursor.is_none())
    {
        return Err(CommunicationsDomainError::MissingMessageScope);
    }
    (-62_135_596_800..=253_402_300_799).contains(&command.observed_at_unix_seconds)
        .then_some(CommunicationSummary {
            evidence_id: command.observation_id, observation_id: command.observation_id,
            source_cursor: command.source_cursor, provider: command.provider, direction: command.direction, kind: command.kind,
            account_cursor: command.account_cursor, conversation_cursor: command.conversation_cursor,
            participant_cursor: command.participant_cursor,
            media_cursor: command.media_cursor,
            reply_to_source_cursor: command.reply_to_source_cursor,
            forward_origin_source_cursor: command.forward_origin_source_cursor,
            body: command.body, body_blob: command.body_blob, body_admission_failure: command.body_admission_failure,
            attachment_descriptor: command.attachment_descriptor, observed_at_unix_seconds: command.observed_at_unix_seconds,
        })
        .ok_or(CommunicationsDomainError::InvalidObservedTime)
}

fn validate_body_admission(command: &RecordCommunicationEvidenceV1) -> Result<(), CommunicationsDomainError> {
    if command.body == hermes_communications_api::CommunicationBodyStateV1::AdmittedBlob {
        let Some(receipt) = command.body_blob.as_ref() else { return Err(CommunicationsDomainError::InvalidAttachmentScope) };
        if receipt.blob_ref.trim().is_empty() || receipt.blob_ref.len() > 512 || !receipt.blob_ref.is_ascii()
            || receipt.reference_id.iter().all(|byte| *byte == 0)
            || !(1..=64 * 1024 * 1024).contains(&receipt.declared_bytes)
            || command.body_admission_failure.is_some()
        { return Err(CommunicationsDomainError::InvalidAttachmentScope); }
    } else if command.body_blob.is_some() {
        return Err(CommunicationsDomainError::InvalidAttachmentScope);
    }
    Ok(())
}

pub fn decide_attachment_safety_transition(
    command: AttachmentSafetyTransitionCommandV1,
) -> Result<AttachmentSafetyTransitionDecisionV1, CommunicationsDomainError> {
    if !(-62_135_596_800..=253_402_300_799).contains(&command.observed_at_unix_seconds) {
        return Err(CommunicationsDomainError::InvalidObservedTime);
    }
    let next_state = command.current_state
        .transition(command.transition)
        .map_err(|_| CommunicationsDomainError::InvalidAttachmentSafetyTransition)?;
    Ok(AttachmentSafetyTransitionDecisionV1 {
        attachment_anchor_id: command.attachment_anchor_id,
        expected_state: command.current_state,
        next_state,
        evidence_id: command.evidence_id,
        observed_at_unix_seconds: command.observed_at_unix_seconds,
    })
}

pub fn canonicalize_communication(
    summary: &CommunicationSummary,
) -> Result<CanonicalCommunicationProjectionV1, CommunicationsDomainError> {
    let account = summary.account_cursor.map(|account_cursor| CanonicalAccountProjectionV1 {
        account_id: CommunicationAccountIdV1::new(identifier(
            b"hermes.communications.account.v1\0",
            &[&account_cursor.bytes()],
        )),
        account_cursor,
        provider: summary.provider,
        observed_at_unix_seconds: summary.observed_at_unix_seconds,
    });
    let conversation = match (summary.account_cursor, summary.conversation_cursor) {
        (Some(account_cursor), Some(conversation_cursor)) => Some(CanonicalConversationProjectionV1 {
            conversation_id: CommunicationConversationIdV1::new(identifier(
                b"hermes.communications.conversation.v1\0",
                &[&account_cursor.bytes(), &conversation_cursor.bytes()],
            )),
            account_cursor,
            conversation_cursor,
            provider: summary.provider,
            observed_at_unix_seconds: summary.observed_at_unix_seconds,
        }),
        _ => None,
    };
    let message = match summary.kind {
        CanonicalCommunicationEvidenceKindV1::EmailMessage
        | CanonicalCommunicationEvidenceKindV1::ChatMessage => {
            let conversation = conversation.as_ref().ok_or(CommunicationsDomainError::MissingMessageScope)?;
            Some(CanonicalMessageProjectionV1 {
                message_id: CommunicationMessageIdV1::new(identifier(
                    b"hermes.communications.message.v1\0",
                    &[&summary.source_cursor.bytes()],
                )),
                conversation_id: conversation.conversation_id,
                source_cursor: summary.source_cursor,
                body: summary.body,
                direction: summary.direction,
                observed_at_unix_seconds: summary.observed_at_unix_seconds,
                mutation: CanonicalMessageMutationV1::Create,
            })
        }
        CanonicalCommunicationEvidenceKindV1::MessageEdited
        | CanonicalCommunicationEvidenceKindV1::ReactionChanged
        | CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged
        | CanonicalCommunicationEvidenceKindV1::MediaChanged => conversation.as_ref().map(|conversation| CanonicalMessageProjectionV1 {
            message_id: CommunicationMessageIdV1::new(identifier(
                b"hermes.communications.message.v1\0",
                &[&summary.source_cursor.bytes()],
            )),
            conversation_id: conversation.conversation_id,
            source_cursor: summary.source_cursor,
            body: summary.body,
            direction: summary.direction,
            observed_at_unix_seconds: summary.observed_at_unix_seconds,
            mutation: CanonicalMessageMutationV1::Update,
        }),
        CanonicalCommunicationEvidenceKindV1::MessageDeleted => conversation.as_ref().map(|conversation| CanonicalMessageProjectionV1 {
            message_id: CommunicationMessageIdV1::new(identifier(
                b"hermes.communications.message.v1\0",
                &[&summary.source_cursor.bytes()],
            )),
            conversation_id: conversation.conversation_id,
            source_cursor: summary.source_cursor,
            body: summary.body,
            direction: summary.direction,
            observed_at_unix_seconds: summary.observed_at_unix_seconds,
            mutation: CanonicalMessageMutationV1::Delete,
        }),
        CanonicalCommunicationEvidenceKindV1::ConversationStateChanged
        | CanonicalCommunicationEvidenceKindV1::ParticipantChanged
        | CanonicalCommunicationEvidenceKindV1::TopicChanged
        | CanonicalCommunicationEvidenceKindV1::TypingChanged => None,
    };
    let participant = match (&conversation, summary.participant_cursor) {
        (Some(conversation), Some(participant_cursor)) => Some(CanonicalObservedParticipantProjectionV1 {
            participant_id: CommunicationParticipantIdV1::new(identifier(
                b"hermes.communications.participant.v1\0",
                &[&conversation.conversation_id.bytes(), &participant_cursor.bytes()],
            )),
            conversation_id: conversation.conversation_id,
            participant_cursor,
            observed_at_unix_seconds: summary.observed_at_unix_seconds,
        }),
        _ => None,
    };
    let attachment_anchor = match (&message, summary.media_cursor) {
        (Some(message), Some(media_cursor)) => Some(CanonicalAttachmentAnchorProjectionV1 {
            attachment_anchor_id: CommunicationAttachmentAnchorIdV1::new(identifier(
                b"hermes.communications.attachment-anchor.v1\0",
                &[&message.message_id.bytes(), &media_cursor.bytes()],
            )),
            message_id: message.message_id,
            media_cursor,
            descriptor: summary.attachment_descriptor.clone(),
            observed_at_unix_seconds: summary.observed_at_unix_seconds,
        }),
        _ => None,
    };
    let message_references = if let Some(message) = message.as_ref()
        && message.mutation == CanonicalMessageMutationV1::Create
    {
        let source_message_id = message.message_id;
        let mut references = Vec::with_capacity(2);
        if let Some(target_source_cursor) = summary.reply_to_source_cursor {
            references.push(CanonicalMessageReferenceProjectionV1 {
                source_message_id,
                target_source_cursor,
                kind: CommunicationMessageReferenceKindV1::Reply,
                observed_at_unix_seconds: summary.observed_at_unix_seconds,
            });
        }
        if let Some(target_source_cursor) = summary.forward_origin_source_cursor {
            references.push(CanonicalMessageReferenceProjectionV1 {
                source_message_id,
                target_source_cursor,
                kind: CommunicationMessageReferenceKindV1::Forward,
                observed_at_unix_seconds: summary.observed_at_unix_seconds,
            });
        }
        references
    } else {
        Vec::new()
    };
    Ok(CanonicalCommunicationProjectionV1 {
        summary: summary.clone(),
        account,
        conversation,
        message,
        participant,
        attachment_anchor,
        message_references,
    })
}

fn identifier(domain: &[u8], values: &[&[u8]]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for value in values {
        hasher.update(value);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16].try_into().expect("fixed SHA-256 prefix length")
}

pub fn convert_client_query_error(error: CommunicationsDomainError) -> CommunicationsClientError {
    match error {
        CommunicationsDomainError::InvalidObservedTime | CommunicationsDomainError::MissingMessageScope => CommunicationsClientError::DraftValidationFailed,
        CommunicationsDomainError::InvalidAttachmentScope => CommunicationsClientError::DraftValidationFailed,
        CommunicationsDomainError::InvalidAttachmentSafetyTransition => CommunicationsClientError::DraftValidationFailed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_communications_api::{
        CommunicationBodyStateV1, CommunicationDirectionV1, CommunicationProviderProvenanceV1,
        CommunicationSourceCursorV1,
    };

    fn cursor(value: u8) -> CommunicationSourceCursorV1 {
        CommunicationSourceCursorV1::new([value; 32])
    }

    #[test]
    fn message_projection_uses_stable_source_and_conversation_identities() {
        let summary = accept_command(RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new([1; 16]),
            source_cursor: cursor(2),
            account_cursor: Some(cursor(3)),
            conversation_cursor: Some(cursor(4)),
            participant_cursor: Some(cursor(5)),
            media_cursor: None,
            reply_to_source_cursor: None,
            forward_origin_source_cursor: None,
            provider: CommunicationProviderProvenanceV1::Telegram,
            direction: CommunicationDirectionV1::Unknown,
            kind: CanonicalCommunicationEvidenceKindV1::ChatMessage,
            body: CommunicationBodyStateV1::MetadataOnly,
            body_blob: None,
            body_admission_failure: None,
            attachment_descriptor: None,
            observed_at_unix_seconds: 1,
        })
        .expect("valid message evidence");

        let first = canonicalize_communication(&summary).expect("projection");
        let second = canonicalize_communication(&summary).expect("projection");

        assert_eq!(first, second);
        assert!(matches!(first.message.as_ref().map(|value| value.mutation), Some(CanonicalMessageMutationV1::Create)));
    }

    #[test]
    fn deleted_message_is_a_typed_transition_not_a_new_message() {
        let summary = accept_command(RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new([1; 16]),
            source_cursor: cursor(2),
            account_cursor: Some(cursor(3)),
            conversation_cursor: Some(cursor(4)),
            participant_cursor: None,
            media_cursor: None,
            reply_to_source_cursor: None,
            forward_origin_source_cursor: None,
            provider: CommunicationProviderProvenanceV1::WhatsAppWeb,
            direction: CommunicationDirectionV1::Unknown,
            kind: CanonicalCommunicationEvidenceKindV1::MessageDeleted,
            body: CommunicationBodyStateV1::MetadataOnly,
            body_blob: None,
            body_admission_failure: None,
            attachment_descriptor: None,
            observed_at_unix_seconds: 1,
        })
        .expect("valid deletion evidence");

        let projection = canonicalize_communication(&summary).expect("projection");

        assert!(matches!(projection.message.as_ref().map(|value| value.mutation), Some(CanonicalMessageMutationV1::Delete)));
    }

    #[test]
    fn message_references_are_typed_and_immutable_projection_inputs() {
        let summary = accept_command(RecordCommunicationEvidenceV1 {
            observation_id: CommunicationObservationIdV1::new([1; 16]),
            source_cursor: cursor(2),
            account_cursor: Some(cursor(3)),
            conversation_cursor: Some(cursor(4)),
            participant_cursor: None,
            media_cursor: None,
            reply_to_source_cursor: Some(cursor(5)),
            forward_origin_source_cursor: Some(cursor(6)),
            provider: CommunicationProviderProvenanceV1::Telegram,
            direction: CommunicationDirectionV1::Unknown,
            kind: CanonicalCommunicationEvidenceKindV1::ChatMessage,
            body: CommunicationBodyStateV1::MetadataOnly,
            body_blob: None,
            body_admission_failure: None,
            attachment_descriptor: None,
            observed_at_unix_seconds: 1,
        })
        .expect("valid message evidence");

        let projection = canonicalize_communication(&summary).expect("projection");

        assert_eq!(projection.message_references.len(), 2);
        assert!(projection.message_references.iter().any(|reference| reference.kind == CommunicationMessageReferenceKindV1::Reply));
        assert!(projection.message_references.iter().any(|reference| reference.kind == CommunicationMessageReferenceKindV1::Forward));
    }
}
