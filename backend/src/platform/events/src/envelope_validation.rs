use prost::Message;

use crate::v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1};

pub const MAX_ENVELOPE_BYTES: usize = 262_144;
pub const MAX_HEADER_BYTES: usize = 16_384;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeValidationError { TooLarge, InvalidEncoding, InvalidMajor, MissingKind, InvalidIdentifier, InvalidContract, InvalidSource, InvalidActor, InvalidFence, InvalidCorrelation, InvalidTimestamp, OversizedHeader }

pub fn decode_envelope_v1(bytes: &[u8]) -> Result<DurableEnvelopeV1, EnvelopeValidationError> {
    if bytes.len() > MAX_ENVELOPE_BYTES { return Err(EnvelopeValidationError::TooLarge); }
    let envelope = DurableEnvelopeV1::decode(bytes).map_err(|_| EnvelopeValidationError::InvalidEncoding)?;
    validate_envelope_v1(&envelope)?;
    Ok(envelope)
}

pub fn validate_envelope_v1(envelope: &DurableEnvelopeV1) -> Result<(), EnvelopeValidationError> {
    if envelope.envelope_major != 1 || envelope.envelope_revision == 0 { return Err(EnvelopeValidationError::InvalidMajor); }
    if envelope.message_id.len() != 16 || envelope.correlation_id.len() != 16 || (!envelope.causation_message_id.is_empty() && envelope.causation_message_id.len() != 16) { return Err(EnvelopeValidationError::InvalidCorrelation); }
    let contract = envelope.contract.as_ref().ok_or(EnvelopeValidationError::InvalidContract)?;
    if !token(&contract.owner, 64) || !token(&contract.name, 64) || contract.major == 0 || contract.revision == 0 || contract.schema_sha256.len() != 32 { return Err(EnvelopeValidationError::InvalidContract); }
    let source = envelope.source.as_ref().ok_or(EnvelopeValidationError::InvalidSource)?;
    if !token(&source.module_id, 128) || source.runtime_instance_id.len() != 16 || source.runtime_generation == 0 { return Err(EnvelopeValidationError::InvalidSource); }
    let actor = envelope.actor.as_ref().ok_or(EnvelopeValidationError::InvalidActor)?;
    if ActorKindV1::try_from(actor.kind).ok().filter(|kind| *kind != ActorKindV1::Unspecified).is_none() || actor.actor_id.is_empty() || actor.actor_id.len() > 64 { return Err(EnvelopeValidationError::InvalidActor); }
    if let Some(fence) = &envelope.source_fence { if FenceKindV1::try_from(fence.kind).ok().filter(|kind| *kind != FenceKindV1::Unspecified).is_none() || fence.scope_id.is_empty() || fence.scope_id.len() > 64 || fence.epoch == 0 { return Err(EnvelopeValidationError::InvalidFence); } }
    let timestamp = envelope.recorded_at.as_ref().ok_or(EnvelopeValidationError::InvalidTimestamp)?;
    if !(0..1_000_000_000).contains(&timestamp.nanos) { return Err(EnvelopeValidationError::InvalidTimestamp); }
    if envelope.semantics.is_none() { return Err(EnvelopeValidationError::MissingKind); }
    let header = envelope.clone();
    let mut header = header; header.payload.clear();
    if header.encoded_len() > MAX_HEADER_BYTES { return Err(EnvelopeValidationError::OversizedHeader); }
    Ok(())
}

fn token(value: &str, max: usize) -> bool { !value.is_empty() && value.len() <= max && value.is_ascii() }
