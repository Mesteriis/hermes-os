use hermes_communications_api::{
    AttachmentDescriptorV1, AttachmentDispositionV1,
    CanonicalCommunicationEvidenceKindV1, CommunicationBodyAdmissionFailureV1,
    CommunicationBodyBlobReferenceV1, CommunicationBodyStateV1, CommunicationDirectionV1,
    CommunicationObservationIdV1, CommunicationProviderProvenanceV1,
    CommunicationSourceCursorV1, RecordCommunicationEvidenceV1,
};
use hermes_communications_domain::{accept_command, canonicalize_communication};
use hermes_communications_ingress::{BodyAvailabilityV1, CommunicationDirectionV1 as IngressDirectionV1, CommunicationEvidenceKindV1, ProviderProvenanceV1};
use hermes_communications_ingress::v1::CommunicationObservationV1;
use hermes_communications_persistence::{
    CommunicationsConsumeOutcomeV1, CommunicationsDurablePersistence,
    CommunicationsPersistenceError,
};
use hermes_events_protocol::{delivery::OutboxRecordV1, v1::{DurableEnvelopeV1, durable_envelope_v1::Semantics}, validation::envelope::decode_envelope_v1};
use hermes_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use prost::Message;

use crate::canonical_outbox::{CanonicalEventContextV1, build_evidence_recorded_outbox_v1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsEventConsumeErrorV1 { InvalidEnvelope, WrongContract, InvalidPayload, DomainRejected, PersistenceRejected }

/// Consumes one already authorized Event Hub delivery. The caller supplies a
/// permit derived by Kernel, so this runtime cannot create or widen a
/// subscription by choosing a subject or budget itself.
pub async fn consume_next_observation_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    canonical_event_context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsDeliveryErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsDeliveryErrorV1::InvalidEnvelope)?;
    let outcome = consume_communication_observation_durable_v1(persistence, &record, canonical_event_context)
        .await
        .map_err(CommunicationsDeliveryErrorV1::Consume)?;
    delivery
        .acknowledge()
        .await
        .map_err(delivery_error)?;
    Ok(outcome)
}

fn delivery_error(_: RuntimePullDeliveryErrorV1) -> CommunicationsDeliveryErrorV1 {
    CommunicationsDeliveryErrorV1::Unavailable
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDeliveryErrorV1 {
    Unavailable,
    InvalidEnvelope,
    Consume(CommunicationsEventConsumeErrorV1),
}

pub async fn consume_communication_observation_durable_v1(
    persistence: &CommunicationsDurablePersistence,
    record: &OutboxRecordV1,
    canonical_event_context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsEventConsumeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidEnvelope)?;
    let summary = accept_command(command_from_envelope(&envelope)?)
        .map_err(|_| CommunicationsEventConsumeErrorV1::DomainRejected)?;
    let projection = canonicalize_communication(&summary)
        .map_err(|_| CommunicationsEventConsumeErrorV1::DomainRejected)?;
    let causation_message_id: [u8; 16] = envelope.message_id.as_slice()
        .try_into()
        .map_err(|_| CommunicationsEventConsumeErrorV1::WrongContract)?;
    let canonical_outbox_record = build_evidence_recorded_outbox_v1(
        &summary,
        causation_message_id,
        canonical_event_context,
    )
    .map_err(|_| CommunicationsEventConsumeErrorV1::DomainRejected)?;
    persistence
        .persist_consumed_observation(
            record,
            projection,
            &canonical_outbox_record,
            canonical_event_context.recorded_at_unix_seconds,
        )
        .await
        .map_err(persistence_error)
}

fn command_from_envelope(envelope: &DurableEnvelopeV1) -> Result<RecordCommunicationEvidenceV1, CommunicationsEventConsumeErrorV1> {
    let contract = envelope.contract.as_ref().ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?;
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else { return Err(CommunicationsEventConsumeErrorV1::WrongContract) };
    if contract.owner != "communications" || contract.name != "communication_observed" || contract.major != 1 || contract.revision != 1 || metadata.observation_id != envelope.message_id || metadata.source_cursor_sha256.len() != 32 { return Err(CommunicationsEventConsumeErrorV1::WrongContract) }
    let payload = CommunicationObservationV1::decode(envelope.payload.as_slice()).map_err(|_| CommunicationsEventConsumeErrorV1::InvalidPayload)?;
    let body = canonical_body(body_from_wire(payload.body)?);
    let body_blob = body_blob_from_wire(payload.body_blob)?;
    let body_admission_failure = body_admission_failure_from_wire(payload.body_admission_failure)?;
    Ok(RecordCommunicationEvidenceV1 {
        observation_id: CommunicationObservationIdV1::new(id16(&metadata.observation_id)?),
        source_cursor: CommunicationSourceCursorV1::new(id32(&metadata.source_cursor_sha256)?),
        account_cursor: optional_cursor(&payload.account_cursor_sha256)?,
        conversation_cursor: optional_cursor(&payload.conversation_cursor_sha256)?,
        participant_cursor: optional_cursor(&payload.participant_cursor_sha256)?,
        media_cursor: optional_cursor(&payload.media_cursor_sha256)?,
        reply_to_source_cursor: optional_cursor(&payload.reply_to_source_cursor_sha256)?,
        forward_origin_source_cursor: optional_cursor(&payload.forward_origin_source_cursor_sha256)?,
        provider: canonical_provider(provider_from_wire(payload.provider)?),
        direction: canonical_direction(direction_from_wire(payload.direction)?),
        kind: canonical_kind(kind_from_wire(payload.kind)?), body, body_blob, body_admission_failure,
        attachment_descriptor: attachment_descriptor_from_wire(payload.attachment_descriptor)?,
        observed_at_unix_seconds: metadata.observed_at.as_ref().ok_or(CommunicationsEventConsumeErrorV1::WrongContract)?.seconds,
    })
}
fn provider_from_wire(value: i32) -> Result<ProviderProvenanceV1, CommunicationsEventConsumeErrorV1> { match value { 1 => Ok(ProviderProvenanceV1::MailImap), 2 => Ok(ProviderProvenanceV1::Telegram), 3 => Ok(ProviderProvenanceV1::WhatsAppWeb), 4 => Ok(ProviderProvenanceV1::MailSmtp), 5 => Ok(ProviderProvenanceV1::Zulip), 6 => Ok(ProviderProvenanceV1::MailGmail), _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload) } }
fn direction_from_wire(value: i32) -> Result<IngressDirectionV1, CommunicationsEventConsumeErrorV1> { match value { 1 => Ok(IngressDirectionV1::Incoming), 2 => Ok(IngressDirectionV1::Outgoing), 3 => Ok(IngressDirectionV1::Unknown), _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload) } }
fn kind_from_wire(value: i32) -> Result<CommunicationEvidenceKindV1, CommunicationsEventConsumeErrorV1> { match value { 1 => Ok(CommunicationEvidenceKindV1::EmailMessage), 2 => Ok(CommunicationEvidenceKindV1::ChatMessage), 3 => Ok(CommunicationEvidenceKindV1::MessageEdited), 4 => Ok(CommunicationEvidenceKindV1::MessageDeleted), 5 => Ok(CommunicationEvidenceKindV1::ReactionChanged), 6 => Ok(CommunicationEvidenceKindV1::DeliveryStateChanged), 7 => Ok(CommunicationEvidenceKindV1::ConversationStateChanged), 8 => Ok(CommunicationEvidenceKindV1::ParticipantChanged), 9 => Ok(CommunicationEvidenceKindV1::MediaChanged), 10 => Ok(CommunicationEvidenceKindV1::TopicChanged), 11 => Ok(CommunicationEvidenceKindV1::TypingChanged), _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload) } }
fn body_from_wire(value: i32) -> Result<BodyAvailabilityV1, CommunicationsEventConsumeErrorV1> { match value { 1 => Ok(BodyAvailabilityV1::MetadataOnly), 2 => Ok(BodyAvailabilityV1::PendingBlob), 3 => Ok(BodyAvailabilityV1::Unavailable), 4 => Ok(BodyAvailabilityV1::AdmittedBlob), _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload) } }
fn body_blob_from_wire(value: Option<hermes_communications_ingress::v1::BodyBlobReceiptV1>) -> Result<Option<CommunicationBodyBlobReferenceV1>, CommunicationsEventConsumeErrorV1> {
    let Some(value) = value else { return Ok(None) };
    let reference_id = id16(&value.reference_id)?;
    let sha256 = id32(&value.sha256)?;
    if value.blob_ref.trim().is_empty() || value.blob_ref.len() > 512 || !value.blob_ref.is_ascii() || reference_id.iter().all(|byte| *byte == 0) || !(1..=64 * 1024 * 1024).contains(&value.declared_bytes) { return Err(CommunicationsEventConsumeErrorV1::InvalidPayload) }
    Ok(Some(CommunicationBodyBlobReferenceV1 { blob_ref: value.blob_ref, reference_id, declared_bytes: value.declared_bytes, sha256 }))
}
fn body_admission_failure_from_wire(value: i32) -> Result<Option<CommunicationBodyAdmissionFailureV1>, CommunicationsEventConsumeErrorV1> { match value { 0 => Ok(None), 1 => Ok(Some(CommunicationBodyAdmissionFailureV1::SourceUnavailable)), 2 => Ok(Some(CommunicationBodyAdmissionFailureV1::SizeLimitExceeded)), 3 => Ok(Some(CommunicationBodyAdmissionFailureV1::IntegrityMismatch)), 4 => Ok(Some(CommunicationBodyAdmissionFailureV1::PolicyRejected)), _ => Err(CommunicationsEventConsumeErrorV1::InvalidPayload) } }
fn attachment_descriptor_from_wire(value: Option<hermes_communications_ingress::v1::AttachmentDescriptorV1>) -> Result<Option<AttachmentDescriptorV1>, CommunicationsEventConsumeErrorV1> {
    let Some(value) = value else { return Ok(None) };
    let disposition = match value.disposition { 1 => AttachmentDispositionV1::Attachment, 2 => AttachmentDispositionV1::Inline, 3 => AttachmentDispositionV1::Unknown, _ => return Err(CommunicationsEventConsumeErrorV1::InvalidPayload) };
    let sha256 = if value.sha256.is_empty() { None } else { Some(id32(&value.sha256)?) };
    AttachmentDescriptorV1::new((!value.filename.is_empty()).then_some(value.filename), value.media_type, value.declared_bytes, sha256, disposition)
        .map(Some)
        .map_err(|_| CommunicationsEventConsumeErrorV1::InvalidPayload)
}
const fn canonical_provider(value: ProviderProvenanceV1) -> CommunicationProviderProvenanceV1 { match value { ProviderProvenanceV1::MailImap => CommunicationProviderProvenanceV1::MailImap, ProviderProvenanceV1::Telegram => CommunicationProviderProvenanceV1::Telegram, ProviderProvenanceV1::WhatsAppWeb => CommunicationProviderProvenanceV1::WhatsAppWeb, ProviderProvenanceV1::MailSmtp => CommunicationProviderProvenanceV1::MailSmtp, ProviderProvenanceV1::Zulip => CommunicationProviderProvenanceV1::Zulip, ProviderProvenanceV1::MailGmail => CommunicationProviderProvenanceV1::MailGmail } }
const fn canonical_direction(value: IngressDirectionV1) -> CommunicationDirectionV1 { match value { IngressDirectionV1::Incoming => CommunicationDirectionV1::Incoming, IngressDirectionV1::Outgoing => CommunicationDirectionV1::Outgoing, IngressDirectionV1::Unknown => CommunicationDirectionV1::Unknown } }
const fn canonical_kind(value: CommunicationEvidenceKindV1) -> CanonicalCommunicationEvidenceKindV1 { match value { CommunicationEvidenceKindV1::EmailMessage => CanonicalCommunicationEvidenceKindV1::EmailMessage, CommunicationEvidenceKindV1::ChatMessage => CanonicalCommunicationEvidenceKindV1::ChatMessage, CommunicationEvidenceKindV1::MessageEdited => CanonicalCommunicationEvidenceKindV1::MessageEdited, CommunicationEvidenceKindV1::MessageDeleted => CanonicalCommunicationEvidenceKindV1::MessageDeleted, CommunicationEvidenceKindV1::ReactionChanged => CanonicalCommunicationEvidenceKindV1::ReactionChanged, CommunicationEvidenceKindV1::DeliveryStateChanged => CanonicalCommunicationEvidenceKindV1::DeliveryStateChanged, CommunicationEvidenceKindV1::ConversationStateChanged => CanonicalCommunicationEvidenceKindV1::ConversationStateChanged, CommunicationEvidenceKindV1::ParticipantChanged => CanonicalCommunicationEvidenceKindV1::ParticipantChanged, CommunicationEvidenceKindV1::MediaChanged => CanonicalCommunicationEvidenceKindV1::MediaChanged, CommunicationEvidenceKindV1::TopicChanged => CanonicalCommunicationEvidenceKindV1::TopicChanged, CommunicationEvidenceKindV1::TypingChanged => CanonicalCommunicationEvidenceKindV1::TypingChanged } }
const fn canonical_body(value: BodyAvailabilityV1) -> CommunicationBodyStateV1 { match value { BodyAvailabilityV1::MetadataOnly => CommunicationBodyStateV1::MetadataOnly, BodyAvailabilityV1::PendingBlob => CommunicationBodyStateV1::PendingBlob, BodyAvailabilityV1::Unavailable => CommunicationBodyStateV1::Unavailable, BodyAvailabilityV1::AdmittedBlob => CommunicationBodyStateV1::AdmittedBlob } }
fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsEventConsumeErrorV1> { value.try_into().map_err(|_| CommunicationsEventConsumeErrorV1::WrongContract) }
fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsEventConsumeErrorV1> { value.try_into().map_err(|_| CommunicationsEventConsumeErrorV1::WrongContract) }
fn optional_cursor(value: &[u8]) -> Result<Option<CommunicationSourceCursorV1>, CommunicationsEventConsumeErrorV1> {
    if value.is_empty() { return Ok(None) }
    Ok(Some(CommunicationSourceCursorV1::new(id32(value)?)))
}
fn persistence_error(_: CommunicationsPersistenceError) -> CommunicationsEventConsumeErrorV1 { CommunicationsEventConsumeErrorV1::PersistenceRejected }

#[cfg(test)]
mod tests {
    use hermes_communications_ingress::{BodyAvailabilityV1, CommunicationDirectionV1, CommunicationEvidenceKindV1, ObservationEnvelopeContextV1, ProviderProvenanceV1, SourceEnvelope, SourceScopeEnvelope, build_observation_outbox_record_v1, new_scoped_communication_observation_draft};
    use super::*;
    #[test]
    fn applies_whatsapp_event_once_without_access_to_provider_locator() {
        let draft = new_scoped_communication_observation_draft("provider-local-id", SourceEnvelope { provider: ProviderProvenanceV1::WhatsAppWeb, external_record_id: "private-chat-and-message".to_owned(), scope: Some(SourceScopeEnvelope { external_account_id: "private-account".to_owned(), external_conversation_id: Some("private-chat".to_owned()), external_participant_id: Some("private-sender".to_owned()), external_media_id: None, external_reply_to_record_id: None, external_forward_origin_record_id: None }) }, CommunicationEvidenceKindV1::ChatMessage, BodyAvailabilityV1::MetadataOnly, CommunicationDirectionV1::Unknown, Some(10)).expect("draft");
        let record = build_observation_outbox_record_v1(&draft, &ObservationEnvelopeContextV1 { runtime_instance_id: "whatsapp_runtime_1".to_owned(), runtime_generation: 1, module_id: "whatsapp-runtime".to_owned(), recorded_at_unix_seconds: 10, recorded_at_nanos: 0 }).expect("record");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let command = command_from_envelope(&envelope).expect("command");
        assert_eq!(command.observation_id, CommunicationObservationIdV1::new(*record.message_id()));
    }

    #[test]
    fn accepts_zulip_provenance_from_the_public_ingress_contract() {
        let draft = new_scoped_communication_observation_draft("provider-local-id", SourceEnvelope { provider: ProviderProvenanceV1::Zulip, external_record_id: "message-7".to_owned(), scope: Some(SourceScopeEnvelope { external_account_id: "account-1".to_owned(), external_conversation_id: Some("stream-1".to_owned()), external_participant_id: None, external_media_id: None, external_reply_to_record_id: None, external_forward_origin_record_id: None }) }, CommunicationEvidenceKindV1::ChatMessage, BodyAvailabilityV1::MetadataOnly, CommunicationDirectionV1::Incoming, Some(10)).expect("draft");
        let record = build_observation_outbox_record_v1(&draft, &ObservationEnvelopeContextV1 { runtime_instance_id: "zulip-runtime-1".to_owned(), runtime_generation: 1, module_id: "zulip-runtime".to_owned(), recorded_at_unix_seconds: 10, recorded_at_nanos: 0 }).expect("record");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let command = command_from_envelope(&envelope).expect("command");
        assert_eq!(command.provider, CommunicationProviderProvenanceV1::Zulip);
    }
}
