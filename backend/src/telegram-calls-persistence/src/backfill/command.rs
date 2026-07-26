use hermes_events_protocol::{
    v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1, durable_envelope_v1::Semantics},
    validation::envelope::validate_envelope_v1,
};
use hermes_scheduler_protocol::{
    SCHEDULER_JOB_DESCRIPTOR_SET_V1,
    v1::{OwnerJobCommandV1, OwnerJobTriggerKindV1},
    validate_owner_job_command_v1,
};
use hermes_telegram_calls_core::{
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1,
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1,
    telegram_calls_realtime_backfill_idempotency_key_v1,
    telegram_calls_realtime_backfill_message_id_v1, telegram_calls_realtime_backfill_run_id_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use super::TelegramCallsBackfillErrorV1;

const MAX_COMMAND_ENVELOPE_BYTES: usize = 256 * 1024;
const TELEGRAM_RUNTIME_MODULE_ID: &str = "hermes-telegram-runtime";
const JOB_EXECUTE_CAPABILITY: &str = "job_execute";

pub(super) struct ParsedBackfillCommandV1 {
    pub(super) envelope_bytes: Vec<u8>,
    pub(super) envelope_sha256: [u8; 32],
    pub(super) run_id: [u8; 16],
    pub(super) message_id: [u8; 16],
    pub(super) accepted_at_unix_millis: i64,
}

pub(super) fn parse_backfill_command_v1(
    envelope_bytes: &[u8],
) -> Result<ParsedBackfillCommandV1, TelegramCallsBackfillErrorV1> {
    if envelope_bytes.is_empty() || envelope_bytes.len() > MAX_COMMAND_ENVELOPE_BYTES {
        return Err(TelegramCallsBackfillErrorV1::InvalidEnvelope);
    }
    let envelope = DurableEnvelopeV1::decode(envelope_bytes)
        .map_err(|_| TelegramCallsBackfillErrorV1::InvalidEnvelope)?;
    validate_envelope_v1(&envelope).map_err(|_| TelegramCallsBackfillErrorV1::InvalidEnvelope)?;
    let command = OwnerJobCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| TelegramCallsBackfillErrorV1::InvalidCommand)?;
    validate_owner_job_command_v1(&command)
        .map_err(|_| TelegramCallsBackfillErrorV1::InvalidCommand)?;
    validate_exact_job(&envelope, &command)?;
    let run_id: [u8; 16] = command
        .job_run_id
        .as_slice()
        .try_into()
        .map_err(|_| TelegramCallsBackfillErrorV1::InvalidCommand)?;
    let message_id: [u8; 16] = envelope
        .message_id
        .as_slice()
        .try_into()
        .map_err(|_| TelegramCallsBackfillErrorV1::InvalidEnvelope)?;
    Ok(ParsedBackfillCommandV1 {
        envelope_bytes: envelope_bytes.to_vec(),
        envelope_sha256: Sha256::digest(envelope_bytes).into(),
        run_id,
        message_id,
        accepted_at_unix_millis: command.accepted_at_unix_millis,
    })
}

fn validate_exact_job(
    envelope: &DurableEnvelopeV1,
    command: &OwnerJobCommandV1,
) -> Result<(), TelegramCallsBackfillErrorV1> {
    let expected_run_id = telegram_calls_realtime_backfill_run_id_v1().bytes();
    let expected_message_id = telegram_calls_realtime_backfill_message_id_v1();
    let expected_schema = Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1);
    let job = command
        .job_kind
        .as_ref()
        .ok_or(TelegramCallsBackfillErrorV1::InvalidCommand)?;
    let lease = command
        .lease
        .as_ref()
        .ok_or(TelegramCallsBackfillErrorV1::InvalidCommand)?;
    if command.job_run_id != expected_run_id
        || envelope.message_id != expected_message_id
        || job.owner != TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1
        || job.name != TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1
        || job.major != u32::from(TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1)
        || command.scope_id != TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1
        || command.trigger_kind != OwnerJobTriggerKindV1::UpgradeReconciliation as i32
        || lease.run_id != expected_run_id
        || lease.epoch != 1
    {
        return Err(TelegramCallsBackfillErrorV1::ContractMismatch);
    }
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(TelegramCallsBackfillErrorV1::InvalidEnvelope)?;
    if contract.owner != TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1
        || contract.name != TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1
        || contract.major != u32::from(TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1)
        || contract.revision != 1
        || contract.schema_sha256 != expected_schema.as_slice()
    {
        return Err(TelegramCallsBackfillErrorV1::ContractMismatch);
    }
    validate_exact_routing(
        envelope,
        command,
        expected_run_id,
        lease.expires_at_unix_millis,
    )
}

fn validate_exact_routing(
    envelope: &DurableEnvelopeV1,
    command: &OwnerJobCommandV1,
    run_id: [u8; 16],
    lease_expires_at_unix_millis: i64,
) -> Result<(), TelegramCallsBackfillErrorV1> {
    let source = envelope
        .source
        .as_ref()
        .ok_or(TelegramCallsBackfillErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(TelegramCallsBackfillErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(TelegramCallsBackfillErrorV1::InvalidEnvelope)?;
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(TelegramCallsBackfillErrorV1::InvalidEnvelope);
    };
    if source.module_id != TELEGRAM_RUNTIME_MODULE_ID
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::System as i32
        || actor.actor_id != TELEGRAM_RUNTIME_MODULE_ID.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != TELEGRAM_RUNTIME_MODULE_ID.as_bytes()
        || fence.epoch != source.runtime_generation
        || envelope.partition_key != TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1.as_bytes()
        || !envelope.causation_message_id.is_empty()
        || envelope.correlation_id != run_id
        || metadata.command_id != run_id
        || metadata.target_capability != JOB_EXECUTE_CAPABILITY
        || metadata.idempotency_key != telegram_calls_realtime_backfill_idempotency_key_v1()
        || metadata.logical_attempt != 1
        || !timestamp_matches(
            envelope.recorded_at.as_ref(),
            command.accepted_at_unix_millis,
        )
        || !timestamp_matches(metadata.deadline.as_ref(), lease_expires_at_unix_millis)
    {
        return Err(TelegramCallsBackfillErrorV1::ContractMismatch);
    }
    Ok(())
}

fn timestamp_matches(timestamp: Option<&prost_types::Timestamp>, unix_millis: i64) -> bool {
    timestamp.is_some_and(|timestamp| {
        timestamp.seconds == unix_millis.div_euclid(1_000)
            && i64::from(timestamp.nanos) == unix_millis.rem_euclid(1_000) * 1_000_000
    })
}
