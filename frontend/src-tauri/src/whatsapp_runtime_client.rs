//! Host-to-WhatsApp-runtime local client. This carries provider contract data only.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use hermes_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1};
use hermes_whatsapp_api::{
    WhatsAppClientResponse, WhatsAppHostBridgeEnvelopeV1, WhatsAppProviderCommand,
    WhatsAppProviderQuery,
    client_wire::{decode_command, encode_query},
    host_bridge::encode_host_bridge_payload,
};
use prost::Message;

const MAX_FRAME_BYTES: usize = 512 * 1024;
const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const MODULE_ID: &str = "hermes-whatsapp-runtime";
const OWNER_ID: &str = "whatsapp";
const CONTRACT_NAME: &str = "whatsapp.client";
const CONTRACT_MAJOR: u32 = 1;
const CONTRACT_REVISION: u32 = 1;

pub fn dispatch_host_observation(
    envelope: WhatsAppHostBridgeEnvelopeV1,
) -> Result<String, String> {
    let expected_provider_event_id = envelope.provider_event_id.clone();
    let request_payload = encode_host_bridge_payload(&envelope)
        .map_err(|error| format!("WhatsApp host observation encoding failed: {error:?}"))?;
    let response = send_payload(request_payload)?;
    if !response.error_code.is_empty() {
        return Err(format!("WhatsApp runtime rejected host observation: {}", response.error_code));
    }
    let response = hermes_whatsapp_api::wire::WhatsAppClientResponseV1::decode(
        response.response_payload.as_slice(),
    )
    .map_err(|error| format!("WhatsApp runtime response payload is invalid: {error}"))?;
    let response = match response.response.ok_or_else(|| "WhatsApp runtime response variant is missing".to_owned())? {
        hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::ObservationAccepted(accepted) => {
            WhatsAppClientResponse::ObservationAccepted { provider_event_id: accepted.provider_event_id }
        }
        _ => return Err("WhatsApp runtime returned a non-observation response".to_owned()),
    };
    match response {
        WhatsAppClientResponse::ObservationAccepted { provider_event_id }
            if provider_event_id == expected_provider_event_id => Ok("accepted".to_owned()),
        WhatsAppClientResponse::ObservationAccepted { .. } => {
            Err("WhatsApp runtime acknowledged a different provider event".to_owned())
        }
        _ => Err("WhatsApp runtime returned a non-observation response".to_owned()),
    }
}

pub fn poll_pending_commands(
    account_id: String,
    host_claim_id: String,
    limit: u32,
) -> Result<Vec<WhatsAppProviderCommand>, String> {
    let payload = encode_query(&WhatsAppProviderQuery::ClaimPendingCommands {
        account_id,
        host_claim_id,
        lease_seconds: 60,
        limit,
    });
    let response = send_payload(payload)?;
    if !response.error_code.is_empty() {
        return Err(format!("WhatsApp runtime rejected command poll: {}", response.error_code));
    }
    let response = hermes_whatsapp_api::wire::WhatsAppClientResponseV1::decode(
        response.response_payload.as_slice(),
    )
    .map_err(|error| format!("WhatsApp command poll response is invalid: {error}"))?;
    match response.response {
        Some(hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::Query(query)) => match query.response {
            Some(hermes_whatsapp_api::wire::whats_app_query_response_v1::Response::Commands(list)) => list
            .command
            .iter()
            .map(|command| decode_command(&command.encode_to_vec()).map_err(|error| format!("WhatsApp command payload is invalid: {error:?}")))
            .collect(),
            _ => Err("WhatsApp runtime returned a non-command-poll response".to_owned()),
        },
        _ => Err("WhatsApp runtime returned a non-command-poll response".to_owned()),
    }
}

pub fn report_command_failure(
    operation_id: String,
    host_claim_id: String,
    reason: String,
) -> Result<(), String> {
    let payload = hermes_whatsapp_api::wire::WhatsAppHostCommandFailureV1 {
        operation_id: operation_id.clone(),
        host_claim_id,
        reason,
    }
    .encode_to_vec();
    let response = send_payload(payload)?;
    if !response.error_code.is_empty() {
        return Err(format!("WhatsApp runtime rejected command failure: {}", response.error_code));
    }
    let response = hermes_whatsapp_api::wire::WhatsAppClientResponseV1::decode(
        response.response_payload.as_slice(),
    )
    .map_err(|error| format!("WhatsApp command failure response is invalid: {error}"))?;
    let recorded = match response.response {
        Some(hermes_whatsapp_api::wire::whats_app_client_response_v1::Response::HostCommandFailureRecorded(value)) => value,
        _ => return Err("WhatsApp runtime returned a non-failure-record response".to_owned()),
    };
    if recorded.operation_id != operation_id {
        return Err("WhatsApp runtime recorded a different operation failure".to_owned());
    }
    Ok(())
}

fn send_payload(payload: Vec<u8>) -> Result<ModuleClientResponseV1, String> {
    let socket_path = std::env::var_os("HERMES_WHATSAPP_RUNTIME_SOCKET")
        .ok_or_else(|| "HERMES_WHATSAPP_RUNTIME_SOCKET is required".to_owned())?;
    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|error| format!("WhatsApp runtime socket is unavailable: {error}"))?;
    let request = ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: MODULE_ID.to_owned(),
        owner_id: OWNER_ID.to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: OWNER_ID.to_owned(),
            name: CONTRACT_NAME.to_owned(),
            major: CONTRACT_MAJOR,
            revision: CONTRACT_REVISION,
            schema_sha256: Vec::new(),
        }),
        request_id: 1,
        request_payload: payload,
    };
    write_frame(&mut stream, &request.encode_to_vec())?;
    ModuleClientResponseV1::decode(read_frame(&mut stream)?.as_slice())
        .map_err(|error| format!("WhatsApp runtime response decoding failed: {error}"))
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err("WhatsApp runtime frame length is invalid".to_owned());
    }
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "WhatsApp runtime frame length is invalid".to_owned())?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    stream
        .write_all(&prefix)
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| format!("WhatsApp runtime request failed: {error}"))
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut length = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| format!("WhatsApp runtime response read failed: {error}"))?;
        length |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            let length = usize::try_from(length)
                .map_err(|_| "WhatsApp runtime frame length is invalid".to_owned())?;
            if length == 0 || length > MAX_FRAME_BYTES {
                return Err("WhatsApp runtime frame length is invalid".to_owned());
            }
            let mut bytes = vec![0_u8; length];
            stream
                .read_exact(&mut bytes)
                .map_err(|error| format!("WhatsApp runtime response read failed: {error}"))?;
            return Ok(bytes);
        }
    }
    Err("WhatsApp runtime frame length is invalid".to_owned())
}
