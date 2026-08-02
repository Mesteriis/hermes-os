use hermes_communications_retained_evidence_replay_contract::{
    validate_communications_replay_command_v1,
    wire::{
        ReplayCommunicationsEvidenceCommandV1, ReplayCommunicationsEvidenceFailureV1,
        ReplayCommunicationsEvidenceOutcomeV1, ReplayCommunicationsEvidenceResultV1,
    },
};
use hermes_communications_retained_evidence_replay_persistence::{
    CommunicationsRetainedEvidenceReplayPersistenceV1, RetainedCommunicationsReplayAuditV1,
    RetainedCommunicationsReplayErrorV1, RetainedCommunicationsReplayPhaseV1,
};
use hermes_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsRetainedEvidenceReplayErrorV1 {
    InvalidCommand,
    StaleRuntimeFence,
    StaleGrantFence,
    Persistence(RetainedCommunicationsReplayErrorV1),
    PublishUnavailable,
}

pub async fn replay_retained_communications_evidence_v1(
    persistence: &CommunicationsRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    original_contract_publish_permit: &RuntimePublishPermitV1,
    command: &ReplayCommunicationsEvidenceCommandV1,
    current_registration_id: &str,
    current_runtime_generation: u64,
    current_grant_epoch: u64,
    logical_attempt: u32,
    recorded_at_unix_seconds: i64,
) -> Result<ReplayCommunicationsEvidenceResultV1, CommunicationsRetainedEvidenceReplayErrorV1> {
    validate_communications_replay_command_v1(command)
        .map_err(|_| CommunicationsRetainedEvidenceReplayErrorV1::InvalidCommand)?;
    if command.producer_registration_id != current_registration_id
        || command.producer_runtime_generation != current_runtime_generation
    {
        return Err(CommunicationsRetainedEvidenceReplayErrorV1::StaleRuntimeFence);
    }
    if command.producer_grant_epoch != current_grant_epoch {
        return Err(CommunicationsRetainedEvidenceReplayErrorV1::StaleGrantFence);
    }
    let operation_id = id16(&command.operation_id)?;
    let actor_sha256 = sha256(&command.owner_device_actor_sha256)?;
    for selected in &command.original_message_ids {
        let message_id = id16(selected)?;
        let retained = persistence
            .retained_attachment_safety_event_by_message_id(message_id)
            .await
            .map_err(CommunicationsRetainedEvidenceReplayErrorV1::Persistence)?;
        let audit = |phase| RetainedCommunicationsReplayAuditV1 {
            operation_id,
            logical_owner_id: command.logical_owner_id.clone(),
            owner_device_actor_sha256: actor_sha256,
            producer_registration_id: current_registration_id.to_owned(),
            producer_runtime_generation: current_runtime_generation,
            producer_grant_epoch: current_grant_epoch,
            logical_attempt,
            original_message_id: message_id,
            original_envelope_sha256: *retained.record.envelope_sha256(),
            phase,
            recorded_at_unix_seconds,
        };
        persistence
            .append_audit(&audit(RetainedCommunicationsReplayPhaseV1::Authorized))
            .await
            .map_err(CommunicationsRetainedEvidenceReplayErrorV1::Persistence)?;
        if connection
            .publish_exact(
                original_contract_publish_permit,
                retained.record.exact_bytes(),
            )
            .await
            .is_err()
        {
            persistence
                .append_audit(&audit(
                    RetainedCommunicationsReplayPhaseV1::PublishUnavailable,
                ))
                .await
                .map_err(CommunicationsRetainedEvidenceReplayErrorV1::Persistence)?;
            return Err(CommunicationsRetainedEvidenceReplayErrorV1::PublishUnavailable);
        }
        persistence
            .append_audit(&audit(RetainedCommunicationsReplayPhaseV1::Published))
            .await
            .map_err(CommunicationsRetainedEvidenceReplayErrorV1::Persistence)?;
    }
    Ok(ReplayCommunicationsEvidenceResultV1 {
        operation_id: command.operation_id.clone(),
        outcome: ReplayCommunicationsEvidenceOutcomeV1::Published as i32,
        original_message_ids: command.original_message_ids.clone(),
        failure: ReplayCommunicationsEvidenceFailureV1::Unspecified as i32,
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsRetainedEvidenceReplayErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsRetainedEvidenceReplayErrorV1::InvalidCommand)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], CommunicationsRetainedEvidenceReplayErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsRetainedEvidenceReplayErrorV1::InvalidCommand)
}
