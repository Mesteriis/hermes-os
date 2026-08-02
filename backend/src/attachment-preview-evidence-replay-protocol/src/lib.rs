#![forbid(unsafe_code)]

use std::collections::HashSet;

pub const PACKAGE: &str = "hermes-retained-evidence-replay-protocol";
pub const RETAINED_EVIDENCE_REPLAY_PROTOCOL_MAJOR_V1: u32 = 1;
pub const RETAINED_EVIDENCE_REPLAY_MAX_MESSAGES_V1: usize = 16;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/hermes.events.replay.v1.rs"));
}

include!(concat!(
    env!("OUT_DIR"),
    "/retained_evidence_replay_schema.rs"
));

pub const RETAINED_EVIDENCE_REPLAY_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/retained-evidence-replay-v1.bin"));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedEvidenceReplayValidationErrorV1 {
    InvalidProtocol,
    InvalidOperationId,
    InvalidLogicalOwner,
    InvalidOwnerDeviceActor,
    InvalidProducerRegistration,
    InvalidRuntimeFence,
    InvalidGrantFence,
    InvalidOriginalContract,
    InvalidMessageSelection,
    DuplicateMessageId,
    InvalidResult,
}

pub fn validate_replay_exact_evidence_command_v1(
    command: &wire::ReplayExactEvidenceCommandV1,
) -> Result<(), RetainedEvidenceReplayValidationErrorV1> {
    if command.protocol_major != RETAINED_EVIDENCE_REPLAY_PROTOCOL_MAJOR_V1 {
        return Err(RetainedEvidenceReplayValidationErrorV1::InvalidProtocol);
    }
    validate_id16(&command.operation_id)
        .map_err(|_| RetainedEvidenceReplayValidationErrorV1::InvalidOperationId)?;
    if !valid_identity(&command.logical_owner_id) {
        return Err(RetainedEvidenceReplayValidationErrorV1::InvalidLogicalOwner);
    }
    if !valid_sha256(&command.owner_device_actor_sha256) {
        return Err(RetainedEvidenceReplayValidationErrorV1::InvalidOwnerDeviceActor);
    }
    if !valid_identity(&command.producer_registration_id) {
        return Err(RetainedEvidenceReplayValidationErrorV1::InvalidProducerRegistration);
    }
    if command.producer_runtime_generation == 0 {
        return Err(RetainedEvidenceReplayValidationErrorV1::InvalidRuntimeFence);
    }
    if command.producer_grant_epoch == 0 {
        return Err(RetainedEvidenceReplayValidationErrorV1::InvalidGrantFence);
    }
    let contract = command
        .original_contract
        .as_ref()
        .ok_or(RetainedEvidenceReplayValidationErrorV1::InvalidOriginalContract)?;
    validate_contract(contract)?;
    validate_message_ids(&command.original_message_ids)
}

pub fn validate_replay_exact_evidence_result_v1(
    result: &wire::ReplayExactEvidenceResultV1,
) -> Result<(), RetainedEvidenceReplayValidationErrorV1> {
    validate_id16(&result.operation_id)
        .map_err(|_| RetainedEvidenceReplayValidationErrorV1::InvalidOperationId)?;
    validate_message_ids(&result.original_message_ids)?;
    use wire::{
        RetainedEvidenceReplayFailureCodeV1 as Failure, RetainedEvidenceReplayOutcomeV1 as Outcome,
    };
    let outcome = Outcome::try_from(result.outcome)
        .map_err(|_| RetainedEvidenceReplayValidationErrorV1::InvalidResult)?;
    let failure = Failure::try_from(result.failure_code)
        .map_err(|_| RetainedEvidenceReplayValidationErrorV1::InvalidResult)?;
    let valid = match outcome {
        Outcome::Published | Outcome::AlreadyPublished => failure == Failure::Unspecified,
        Outcome::Rejected | Outcome::Unavailable => failure != Failure::Unspecified,
        Outcome::Unspecified => false,
    };
    valid
        .then_some(())
        .ok_or(RetainedEvidenceReplayValidationErrorV1::InvalidResult)
}

fn validate_contract(
    contract: &wire::RetainedEvidenceContractRefV1,
) -> Result<(), RetainedEvidenceReplayValidationErrorV1> {
    if !valid_identity(&contract.owner)
        || !valid_contract_name(&contract.name)
        || contract.major == 0
        || contract.revision == 0
        || !valid_sha256(&contract.schema_sha256)
    {
        return Err(RetainedEvidenceReplayValidationErrorV1::InvalidOriginalContract);
    }
    Ok(())
}

fn validate_message_ids(
    message_ids: &[Vec<u8>],
) -> Result<(), RetainedEvidenceReplayValidationErrorV1> {
    if message_ids.is_empty() || message_ids.len() > RETAINED_EVIDENCE_REPLAY_MAX_MESSAGES_V1 {
        return Err(RetainedEvidenceReplayValidationErrorV1::InvalidMessageSelection);
    }
    let mut unique = HashSet::with_capacity(message_ids.len());
    for message_id in message_ids {
        validate_id16(message_id)
            .map_err(|_| RetainedEvidenceReplayValidationErrorV1::InvalidMessageSelection)?;
        if !unique.insert(message_id.as_slice()) {
            return Err(RetainedEvidenceReplayValidationErrorV1::DuplicateMessageId);
        }
    }
    Ok(())
}

fn validate_id16(value: &[u8]) -> Result<(), ()> {
    (value.len() == 16 && value.iter().any(|byte| *byte != 0))
        .then_some(())
        .ok_or(())
}

fn valid_sha256(value: &[u8]) -> bool {
    value.len() == 32 && value.iter().any(|byte| *byte != 0)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn valid_contract_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::{wire::*, *};

    fn command() -> ReplayExactEvidenceCommandV1 {
        ReplayExactEvidenceCommandV1 {
            protocol_major: 1,
            operation_id: vec![1; 16],
            logical_owner_id: "attachment_preview".to_owned(),
            owner_device_actor_sha256: vec![2; 32],
            producer_registration_id: "communications-runtime".to_owned(),
            producer_runtime_generation: 7,
            producer_grant_epoch: 9,
            original_contract: Some(RetainedEvidenceContractRefV1 {
                owner: "communications".to_owned(),
                name: "communication_attachment_safety_state_changed".to_owned(),
                major: 1,
                revision: 1,
                schema_sha256: vec![3; 32],
            }),
            original_message_ids: vec![vec![4; 16], vec![5; 16]],
        }
    }

    #[test]
    fn accepts_only_bounded_exact_producer_selection() {
        assert_eq!(
            validate_replay_exact_evidence_command_v1(&command()),
            Ok(())
        );

        let mut duplicate = command();
        duplicate.original_message_ids[1] = duplicate.original_message_ids[0].clone();
        assert_eq!(
            validate_replay_exact_evidence_command_v1(&duplicate),
            Err(RetainedEvidenceReplayValidationErrorV1::DuplicateMessageId)
        );

        let mut unbounded = command();
        unbounded.original_message_ids =
            vec![vec![7; 16]; RETAINED_EVIDENCE_REPLAY_MAX_MESSAGES_V1 + 1];
        assert_eq!(
            validate_replay_exact_evidence_command_v1(&unbounded),
            Err(RetainedEvidenceReplayValidationErrorV1::InvalidMessageSelection)
        );
    }

    #[test]
    fn fences_owner_device_runtime_and_grant() {
        let mut missing_device = command();
        missing_device.owner_device_actor_sha256.clear();
        assert_eq!(
            validate_replay_exact_evidence_command_v1(&missing_device),
            Err(RetainedEvidenceReplayValidationErrorV1::InvalidOwnerDeviceActor)
        );

        let mut stale_runtime = command();
        stale_runtime.producer_runtime_generation = 0;
        assert_eq!(
            validate_replay_exact_evidence_command_v1(&stale_runtime),
            Err(RetainedEvidenceReplayValidationErrorV1::InvalidRuntimeFence)
        );

        let mut stale_grant = command();
        stale_grant.producer_grant_epoch = 0;
        assert_eq!(
            validate_replay_exact_evidence_command_v1(&stale_grant),
            Err(RetainedEvidenceReplayValidationErrorV1::InvalidGrantFence)
        );
    }

    #[test]
    fn result_requires_sanitized_terminal_outcome() {
        let success = ReplayExactEvidenceResultV1 {
            operation_id: vec![1; 16],
            outcome: RetainedEvidenceReplayOutcomeV1::Published as i32,
            original_message_ids: vec![vec![2; 16]],
            failure_code: RetainedEvidenceReplayFailureCodeV1::Unspecified as i32,
        };
        assert_eq!(validate_replay_exact_evidence_result_v1(&success), Ok(()));

        let mut invalid = success;
        invalid.outcome = RetainedEvidenceReplayOutcomeV1::Rejected as i32;
        assert_eq!(
            validate_replay_exact_evidence_result_v1(&invalid),
            Err(RetainedEvidenceReplayValidationErrorV1::InvalidResult)
        );
    }

    #[test]
    fn schema_exposes_no_subject_query_or_payload_bytes() {
        let source =
            include_str!("../proto/hermes/events/replay/v1/retained_evidence_replay.proto");
        assert!(!source.contains("subject"));
        assert!(!source.contains("predicate"));
        assert!(!source.contains("payload_bytes"));
        assert!(!source.contains("map<"));
    }
}
