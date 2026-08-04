use prost::Message;

use crate::{
    HOST_PROTOCOL_MAJOR_V1, HOST_PROTOCOL_REVISION_V1, MAX_AUDIO_BYTES_V1,
    wire::{
        DesktopRecordingHostCommandLeaseV1, DesktopRecordingHostCommandV1,
        DesktopRecordingHostHandshakeAcceptedV1, DesktopRecordingHostHandshakeV1,
        DesktopRecordingHostObservationAcceptedV1, DesktopRecordingHostOperationV1,
        desktop_recording_host_observation_v1::Observation,
        desktop_recording_host_operation_v1::Operation,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostBridgeContractErrorV1 {
    InvalidProtocol,
    InvalidPayload,
    PayloadTooLarge,
}

pub fn decode_handshake_v1(
    bytes: &[u8],
) -> Result<DesktopRecordingHostHandshakeV1, HostBridgeContractErrorV1> {
    let value = DesktopRecordingHostHandshakeV1::decode(bytes)
        .map_err(|_| HostBridgeContractErrorV1::InvalidPayload)?;
    if value.protocol_major != HOST_PROTOCOL_MAJOR_V1
        || value.protocol_revision != HOST_PROTOCOL_REVISION_V1
        || value.route_binding_sha256.len() != 32
    {
        return Err(HostBridgeContractErrorV1::InvalidProtocol);
    }
    Ok(value)
}

#[must_use]
pub fn encode_handshake_accepted_v1() -> Vec<u8> {
    DesktopRecordingHostHandshakeAcceptedV1 {
        protocol_major: HOST_PROTOCOL_MAJOR_V1,
        protocol_revision: HOST_PROTOCOL_REVISION_V1,
    }
    .encode_to_vec()
}

pub fn decode_operation_v1(
    bytes: &[u8],
) -> Result<DesktopRecordingHostOperationV1, HostBridgeContractErrorV1> {
    if bytes.is_empty() || bytes.len() > MAX_AUDIO_BYTES_V1 + 16 * 1024 {
        return Err(HostBridgeContractErrorV1::PayloadTooLarge);
    }
    let value = DesktopRecordingHostOperationV1::decode(bytes)
        .map_err(|_| HostBridgeContractErrorV1::InvalidPayload)?;
    match value.operation.as_ref() {
        Some(Operation::ClaimCommands(claim))
            if exact_id(&claim.host_claim_id)
                && (1..=60).contains(&claim.lease_seconds)
                && (1..=16).contains(&claim.limit) => {}
        Some(Operation::Observation(observation)) => match observation.observation.as_ref() {
            Some(Observation::CaptureStarted(started))
                if exact_id(&started.command_id)
                    && exact_id(&started.host_claim_id)
                    && exact_id(&started.challenge_id)
                    && exact_id(&started.recording_evidence_id)
                    && started.started_at_unix_ms > 0
                    && started.os_permission_revision > 0 => {}
            Some(Observation::CaptureCompleted(completed))
                if exact_id(&completed.command_id)
                    && exact_id(&completed.host_claim_id)
                    && exact_id(&completed.challenge_id)
                    && exact_id(&completed.recording_evidence_id)
                    && completed.started_at_unix_ms > 0
                    && completed.ended_at_unix_ms > completed.started_at_unix_ms
                    && !completed.canonical_wav_bytes.is_empty()
                    && completed.canonical_wav_bytes.len() <= MAX_AUDIO_BYTES_V1
                    && completed.audio_sha256.len() == 32 => {}
            Some(Observation::CaptureRejected(rejected))
                if exact_id(&rejected.command_id)
                    && exact_id(&rejected.host_claim_id)
                    && exact_id(&rejected.challenge_id)
                    && exact_id(&rejected.recording_evidence_id)
                    && valid_code(&rejected.rejection_code) => {}
            _ => return Err(HostBridgeContractErrorV1::InvalidPayload),
        },
        _ => return Err(HostBridgeContractErrorV1::InvalidPayload),
    }
    Ok(value)
}

fn exact_id(value: &[u8]) -> bool {
    value.len() == 16 && value.iter().any(|byte| *byte != 0)
}

fn valid_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

#[must_use]
pub fn encode_command_lease_v1(commands: Vec<DesktopRecordingHostCommandV1>) -> Vec<u8> {
    DesktopRecordingHostCommandLeaseV1 { commands }.encode_to_vec()
}

#[must_use]
pub fn encode_observation_accepted_v1(recording_evidence_id: [u8; 16], revision: u64) -> Vec<u8> {
    DesktopRecordingHostObservationAcceptedV1 {
        recording_evidence_id: recording_evidence_id.to_vec(),
        recording_revision: revision,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handshake_is_exact_and_route_bound() {
        let bytes = DesktopRecordingHostHandshakeV1 {
            protocol_major: 1,
            protocol_revision: 1,
            route_binding_sha256: vec![7; 32],
        }
        .encode_to_vec();
        assert_eq!(
            decode_handshake_v1(&bytes)
                .expect("handshake")
                .route_binding_sha256
                .len(),
            32
        );
    }
}
