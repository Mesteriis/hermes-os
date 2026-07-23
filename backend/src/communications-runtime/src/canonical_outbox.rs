//! Exact canonical Communications event records for downstream owner consumers.

use hermes_communications_api::{
    COMMUNICATION_EVIDENCE_SCHEMA_SHA256, CanonicalCommunicationEvidenceKindV1,
    CommunicationBodyStateV1, CommunicationDirectionV1, CommunicationProviderProvenanceV1, CommunicationSummary,
    wire::CommunicationEvidenceRecordedV1,
};
use hermes_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1,
        FenceKindV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalEventContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalOutboxBuildErrorV1 {
    InvalidContext,
    InvalidEnvelope,
}

pub fn build_evidence_recorded_outbox_v1(
    summary: &CommunicationSummary,
    causation_message_id: [u8; 16],
    context: &CanonicalEventContextV1,
) -> Result<OutboxRecordV1, CanonicalOutboxBuildErrorV1> {
    if !valid_context(context) {
        return Err(CanonicalOutboxBuildErrorV1::InvalidContext);
    }
    let message_id = identifier(b"hermes.communications.evidence-recorded.v1\0", summary.evidence_id.bytes());
    let correlation_id = identifier(b"hermes.communications.evidence-correlation.v1\0", summary.evidence_id.bytes());
    let recorded_at = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let occurred_at = Timestamp {
        seconds: summary.observed_at_unix_seconds,
        nanos: 0,
    };
    let payload = CommunicationEvidenceRecordedV1 {
        evidence_id: summary.evidence_id.bytes().to_vec(),
        source_cursor_sha256: summary.source_cursor.bytes().to_vec(),
        account_cursor_sha256: summary.account_cursor.map_or_else(Vec::new, |value| value.bytes().to_vec()),
        conversation_cursor_sha256: summary.conversation_cursor.map_or_else(Vec::new, |value| value.bytes().to_vec()),
        participant_cursor_sha256: summary.participant_cursor.map_or_else(Vec::new, |value| value.bytes().to_vec()),
        media_cursor_sha256: summary.media_cursor.map_or_else(Vec::new, |value| value.bytes().to_vec()),
        reply_to_source_cursor_sha256: summary.reply_to_source_cursor.map_or_else(Vec::new, |value| value.bytes().to_vec()),
        forward_origin_source_cursor_sha256: summary.forward_origin_source_cursor.map_or_else(Vec::new, |value| value.bytes().to_vec()),
        provider: provider_value(summary.provider),
        kind: kind_value(summary.kind),
        body: body_value(summary.body),
        direction: direction_value(summary.direction),
        observed_at_unix_seconds: summary.observed_at_unix_seconds,
    }
    .encode_to_vec();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: "communications".to_owned(),
            name: "communication_evidence_recorded".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: COMMUNICATION_EVIDENCE_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: "communications-runtime".to_owned(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(recorded_at),
        partition_key: summary.source_cursor.bytes().to_vec(),
        causation_message_id: causation_message_id.to_vec(),
        correlation_id: correlation_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: b"communications-runtime".to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: b"communications-runtime".to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Event(EventMetadataV1 {
            occurred_at: Some(occurred_at),
        })),
        payload,
    };
    validate_envelope_v1(&envelope).map_err(|_| CanonicalOutboxBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(|_| CanonicalOutboxBuildErrorV1::InvalidEnvelope)
}

fn valid_context(context: &CanonicalEventContextV1) -> bool {
    context.runtime_generation != 0
        && !context.runtime_instance_id.is_empty()
        && context.runtime_instance_id.len() <= 64
        && context.runtime_instance_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
        && (-62_135_596_800..=253_402_300_799).contains(&context.recorded_at_unix_seconds)
        && (0..1_000_000_000).contains(&context.recorded_at_nanos)
}

fn identifier(domain: &[u8], evidence_id: [u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(evidence_id);
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16].try_into().expect("fixed SHA-256 prefix length")
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.communications.runtime-source.v1\0");
    hasher.update(runtime_instance_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16].try_into().expect("fixed SHA-256 prefix length")
}

const fn provider_value(value: CommunicationProviderProvenanceV1) -> i32 {
    match value {
        CommunicationProviderProvenanceV1::MailImap => 1,
        CommunicationProviderProvenanceV1::Telegram => 2,
        CommunicationProviderProvenanceV1::WhatsAppWeb => 3,
        CommunicationProviderProvenanceV1::MailSmtp => 4,
        CommunicationProviderProvenanceV1::Zulip => 5,
        CommunicationProviderProvenanceV1::MailGmail => 6,
    }
}

const fn direction_value(value: CommunicationDirectionV1) -> i32 {
    match value {
        CommunicationDirectionV1::Incoming => 1,
        CommunicationDirectionV1::Outgoing => 2,
        CommunicationDirectionV1::Unknown => 3,
    }
}

const fn kind_value(value: CanonicalCommunicationEvidenceKindV1) -> i32 {
    match value {
        CanonicalCommunicationEvidenceKindV1::EmailMessage => 1,
        CanonicalCommunicationEvidenceKindV1::ChatMessage => 2,
        CanonicalCommunicationEvidenceKindV1::MessageEdited => 3,
        CanonicalCommunicationEvidenceKindV1::MessageDeleted => 4,
        CanonicalCommunicationEvidenceKindV1::ReactionChanged => 5,
        CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged => 6,
        CanonicalCommunicationEvidenceKindV1::ConversationStateChanged => 7,
        CanonicalCommunicationEvidenceKindV1::ParticipantChanged => 8,
        CanonicalCommunicationEvidenceKindV1::MediaChanged => 9,
        CanonicalCommunicationEvidenceKindV1::TopicChanged => 10,
        CanonicalCommunicationEvidenceKindV1::TypingChanged => 11,
    }
}

const fn body_value(value: CommunicationBodyStateV1) -> i32 {
    match value {
        CommunicationBodyStateV1::MetadataOnly => 1,
        CommunicationBodyStateV1::PendingBlob => 2,
        CommunicationBodyStateV1::Unavailable => 3,
        CommunicationBodyStateV1::AdmittedBlob => 4,
    }
}
