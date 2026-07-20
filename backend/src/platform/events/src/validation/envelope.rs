use prost::Message;
use prost_types::Timestamp;

use crate::v1::{
    AckDispositionV1, AckStageV1, ActorKindV1, DurableEnvelopeV1, FenceKindV1, ResultOutcomeV1,
    durable_envelope_v1::Semantics,
};

pub const MAX_ENVELOPE_BYTES: usize = 262_144;
pub const MAX_HEADER_BYTES: usize = 16_384;
const MAX_PARTITION_KEY_BYTES: usize = 256;
const MAX_IDEMPOTENCY_KEY_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeValidationError {
    TooLarge,
    InvalidEncoding,
    InvalidMajor,
    MissingKind,
    InvalidIdentifier,
    InvalidContract,
    InvalidSource,
    InvalidActor,
    InvalidFence,
    InvalidCorrelation,
    InvalidTimestamp,
    InvalidTrace,
    InvalidPartition,
    InvalidMetadata,
    OversizedHeader,
}

pub fn decode_envelope_v1(bytes: &[u8]) -> Result<DurableEnvelopeV1, EnvelopeValidationError> {
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(EnvelopeValidationError::TooLarge);
    }
    let envelope =
        DurableEnvelopeV1::decode(bytes).map_err(|_| EnvelopeValidationError::InvalidEncoding)?;
    validate_envelope_v1(&envelope)?;
    Ok(envelope)
}

pub fn validate_envelope_v1(envelope: &DurableEnvelopeV1) -> Result<(), EnvelopeValidationError> {
    validate_version_and_ids(envelope)?;
    validate_contract_and_source(envelope)?;
    validate_actor_trace_and_fence(envelope)?;
    validate_semantics(envelope.semantics.as_ref())?;
    let mut header = envelope.clone();
    header.payload.clear();
    if header.encoded_len() > MAX_HEADER_BYTES {
        return Err(EnvelopeValidationError::OversizedHeader);
    }
    Ok(())
}

fn validate_version_and_ids(envelope: &DurableEnvelopeV1) -> Result<(), EnvelopeValidationError> {
    if envelope.envelope_major != 1 || envelope.envelope_revision == 0 {
        return Err(EnvelopeValidationError::InvalidMajor);
    }
    if !exact_id(&envelope.message_id)
        || !exact_id(&envelope.correlation_id)
        || (!envelope.causation_message_id.is_empty() && !exact_id(&envelope.causation_message_id))
    {
        return Err(EnvelopeValidationError::InvalidCorrelation);
    }
    if envelope.partition_key.is_empty() || envelope.partition_key.len() > MAX_PARTITION_KEY_BYTES {
        return Err(EnvelopeValidationError::InvalidPartition);
    }
    validate_timestamp(envelope.recorded_at.as_ref())
}

fn validate_contract_and_source(
    envelope: &DurableEnvelopeV1,
) -> Result<(), EnvelopeValidationError> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(EnvelopeValidationError::InvalidContract)?;
    if !token(&contract.owner, 64)
        || !token(&contract.name, 64)
        || contract.major == 0
        || contract.revision == 0
        || contract.schema_sha256.len() != 32
    {
        return Err(EnvelopeValidationError::InvalidContract);
    }
    let source = envelope
        .source
        .as_ref()
        .ok_or(EnvelopeValidationError::InvalidSource)?;
    if !token(&source.module_id, 128)
        || !exact_id(&source.runtime_instance_id)
        || source.runtime_generation == 0
    {
        return Err(EnvelopeValidationError::InvalidSource);
    }
    Ok(())
}

fn validate_actor_trace_and_fence(
    envelope: &DurableEnvelopeV1,
) -> Result<(), EnvelopeValidationError> {
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(EnvelopeValidationError::InvalidActor)?;
    if !valid_enum::<ActorKindV1>(actor.kind)
        || actor.actor_id.is_empty()
        || actor.actor_id.len() > 64
    {
        return Err(EnvelopeValidationError::InvalidActor);
    }
    if let Some(trace) = &envelope.trace
        && (trace.trace_id.len() != 16 || trace.parent_span_id.len() != 8 || trace.trace_flags > 1)
    {
        return Err(EnvelopeValidationError::InvalidTrace);
    }
    if let Some(fence) = &envelope.source_fence
        && (!valid_enum::<FenceKindV1>(fence.kind)
            || fence.scope_id.is_empty()
            || fence.scope_id.len() > 64
            || fence.epoch == 0)
    {
        return Err(EnvelopeValidationError::InvalidFence);
    }
    Ok(())
}

fn validate_semantics(semantics: Option<&Semantics>) -> Result<(), EnvelopeValidationError> {
    let valid = match semantics.ok_or(EnvelopeValidationError::MissingKind)? {
        Semantics::Command(value) => {
            exact_id(&value.command_id)
                && token(&value.target_capability, 128)
                && !value.idempotency_key.is_empty()
                && value.idempotency_key.len() <= MAX_IDEMPOTENCY_KEY_BYTES
                && value.logical_attempt > 0
                && valid_timestamp(value.deadline.as_ref())
        }
        Semantics::Event(value) => valid_timestamp(value.occurred_at.as_ref()),
        Semantics::Observation(value) => {
            exact_id(&value.observation_id)
                && valid_timestamp(value.observed_at.as_ref())
                && value
                    .occurred_at
                    .as_ref()
                    .is_none_or(|time| valid_timestamp(Some(time)))
                && value.source_cursor_sha256.len() == 32
                && value.source_sequence.is_none_or(|sequence| sequence > 0)
        }
        Semantics::Result(value) => {
            exact_id(&value.command_id)
                && exact_id(&value.command_message_id)
                && valid_enum::<ResultOutcomeV1>(value.outcome)
                && valid_timestamp(value.completed_at.as_ref())
                && value.execution_attempt > 0
        }
        Semantics::Ack(value) => {
            exact_id(&value.acknowledged_message_id)
                && valid_enum::<AckStageV1>(value.stage)
                && valid_enum::<AckDispositionV1>(value.disposition)
                && valid_timestamp(value.acknowledged_at.as_ref())
        }
    };
    valid
        .then_some(())
        .ok_or(EnvelopeValidationError::InvalidMetadata)
}

fn validate_timestamp(timestamp: Option<&Timestamp>) -> Result<(), EnvelopeValidationError> {
    valid_timestamp(timestamp)
        .then_some(())
        .ok_or(EnvelopeValidationError::InvalidTimestamp)
}

fn valid_timestamp(timestamp: Option<&Timestamp>) -> bool {
    timestamp.is_some_and(|value| {
        (-62_135_596_800..=253_402_300_799).contains(&value.seconds)
            && (0..1_000_000_000).contains(&value.nanos)
    })
}

fn valid_enum<T>(value: i32) -> bool
where
    T: TryFrom<i32> + PartialEq + Default,
{
    T::try_from(value)
        .ok()
        .is_some_and(|item| item != T::default())
}

fn exact_id(value: &[u8]) -> bool {
    value.len() == 16
}

fn token(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && value.is_ascii()
}
