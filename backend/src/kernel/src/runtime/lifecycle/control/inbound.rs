//! Bounded inbound Vault route messages on a verified managed-runtime channel.

use std::io::ErrorKind;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

use hermes_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeEventCredentialDeliveryV1, ManagedRuntimeEventCredentialRequestV1,
    ManagedRuntimeProviderCredentialDeliveryV1, ManagedRuntimeProviderCredentialRequestV1,
    ManagedRuntimeOwnerDerivedKeyDeliveryV1, ManagedRuntimeOwnerDerivedKeyRequestV1,
    ManagedRuntimeBlobSessionDeliveryV1, ManagedRuntimeBlobSessionRequestV1,
    ManagedRuntimeReadyRequestV1, ManagedRuntimeVaultRouteRequestV1,
    ManagedRuntimeVaultRouteResponseV1, VaultCiphertextResponseV1, VaultCiphertextRouteV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use super::{MAX_FRAME_BYTES, read_frame, write_frame};

const VAULT_ROUTE_FIELD_TAG: u8 = 0x0a;
const READY_FIELD_TAG: u8 = 0x12;
const EVENT_CREDENTIAL_FIELD_TAG: u8 = 0x1a;
const PROVIDER_CREDENTIAL_FIELD_TAG: u8 = 0x22;
const BLOB_SESSION_FIELD_TAG: u8 = 0x32;
const OWNER_DERIVED_KEY_FIELD_TAG: u8 = 0x3a;

pub(crate) fn try_receive_vault_route(
    channel: &mut UnixStream,
) -> Result<Option<VaultCiphertextRouteV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&VAULT_ROUTE_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeVaultRouteRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime Vault route is invalid".to_owned())?;
    let Some(route) = request.route else {
        return Ok(None);
    };
    read_frame(channel)?;
    Ok(Some(route))
}

pub(crate) fn try_receive_ready(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeReadyRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&READY_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime ready signal is invalid".to_owned())?;
    let Some(Operation::Ready(ready)) = request.operation else {
        return Ok(None);
    };
    read_frame(channel)?;
    Ok(Some(ready))
}

pub(crate) fn try_receive_event_credential(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeEventCredentialRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&EVENT_CREDENTIAL_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime event credential request is invalid".to_owned())?;
    let Some(Operation::IssueEventCredential(value)) = request.operation else {
        return Err("managed runtime event credential request is invalid".to_owned());
    };
    valid_event_credential_request(&value)
        .then_some(())
        .ok_or_else(|| "managed runtime event credential request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(value))
}

pub(crate) fn respond_vault_route(
    channel: &mut UnixStream,
    result: Result<VaultCiphertextResponseV1, String>,
) -> Result<(), String> {
    let response = match result {
        Ok(response) => ManagedRuntimeVaultRouteResponseV1 {
            response: Some(response),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeVaultRouteResponseV1 {
            response: None,
            error_code: "managed_vault_route_denied".to_owned(),
        },
    };
    write_frame(channel, &response.encode_to_vec())
}

pub(crate) fn respond_event_credential(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeEventCredentialDeliveryV1, String>,
) -> Result<(), String> {
    let response = match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::EventCredentialDelivery(delivery)),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_event_credential_denied".to_owned(),
        },
    };
    write_frame(channel, &response.encode_to_vec())
}

pub(crate) fn try_receive_provider_credential(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeProviderCredentialRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&PROVIDER_CREDENTIAL_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime provider credential request is invalid".to_owned())?;
    let Some(Operation::IssueProviderCredential(value)) = request.operation else {
        return Err("managed runtime provider credential request is invalid".to_owned());
    };
    valid_provider_credential_request(&value)
        .then_some(())
        .ok_or_else(|| "managed runtime provider credential request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(value))
}

pub(crate) fn respond_provider_credential(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeProviderCredentialDeliveryV1, String>,
) -> Result<(), String> {
    let response = match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::ProviderCredentialDelivery(delivery)),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_provider_credential_denied".to_owned(),
        },
    };
    write_frame(channel, &response.encode_to_vec())
}

pub(crate) fn try_receive_owner_derived_key(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeOwnerDerivedKeyRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&OWNER_DERIVED_KEY_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime owner-derived key request is invalid".to_owned())?;
    let Some(Operation::IssueOwnerDerivedKey(value)) = request.operation else {
        return Err("managed runtime owner-derived key request is invalid".to_owned());
    };
    valid_owner_derived_key_request(&value)
        .then_some(())
        .ok_or_else(|| "managed runtime owner-derived key request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(value))
}

pub(crate) fn respond_owner_derived_key(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeOwnerDerivedKeyDeliveryV1, String>,
) -> Result<(), String> {
    let response = match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::OwnerDerivedKeyDelivery(delivery)),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_owner_derived_key_denied".to_owned(),
        },
    };
    write_frame(channel, &response.encode_to_vec())
}

pub(crate) fn try_receive_blob_session(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeBlobSessionRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&BLOB_SESSION_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime Blob session request is invalid".to_owned())?;
    let Some(Operation::IssueBlobSession(value)) = request.operation else {
        return Err("managed runtime Blob session request is invalid".to_owned());
    };
    crate::platform::blob::session::valid_request(&value)
        .then_some(())
        .ok_or_else(|| "managed runtime Blob session request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(value))
}

pub(crate) fn respond_blob_session(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeBlobSessionDeliveryV1, String>,
) -> Result<(), String> {
    let response = match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::BlobSessionDelivery(delivery)),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_blob_session_denied".to_owned(),
        },
    };
    write_frame(channel, &response.encode_to_vec())
}

fn peek_complete_frame(channel: &mut UnixStream) -> Result<Option<Vec<u8>>, String> {
    let mut header = [0_u8; 5];
    let header_length = match peek(channel, &mut header) {
        Ok(length) => length,
        Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let Some((prefix_length, payload_length)) = decode_length(&header[..header_length])? else {
        return Ok(None);
    };
    let total_length = prefix_length
        .checked_add(payload_length)
        .ok_or_else(|| "managed runtime Vault route is invalid".to_owned())?;
    let mut frame = vec![0_u8; total_length];
    let available = match peek(channel, &mut frame) {
        Ok(length) => length,
        Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if available < total_length {
        return Ok(None);
    }
    // `read_frame` returns the protobuf payload without its varint length prefix.
    // Keep the non-blocking probe equivalent so callers can inspect protobuf field
    // tags without accidentally treating the frame prefix as message content.
    Ok(Some(frame[prefix_length..].to_vec()))
}

fn peek(channel: &UnixStream, bytes: &mut [u8]) -> std::io::Result<usize> {
    let length = unsafe {
        libc::recv(
            channel.as_raw_fd(),
            bytes.as_mut_ptr().cast(),
            bytes.len(),
            libc::MSG_PEEK | libc::MSG_DONTWAIT,
        )
    };
    if length < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        usize::try_from(length).map_err(|_| std::io::Error::other("invalid socket frame length"))
    }
}

fn decode_length(bytes: &[u8]) -> Result<Option<(usize, usize)>, String> {
    let mut value = 0_u64;
    for (index, byte) in bytes.iter().copied().enumerate() {
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            let payload_length = usize::try_from(value)
                .map_err(|_| "managed runtime Vault route is invalid".to_owned())?;
            if payload_length == 0 || payload_length > MAX_FRAME_BYTES {
                return Err("managed runtime Vault route is invalid".to_owned());
            }
            return Ok(Some((index + 1, payload_length)));
        }
    }
    if bytes.len() == 5 {
        return Err("managed runtime Vault route is invalid".to_owned());
    }
    Ok(None)
}

fn valid_event_credential_request(value: &ManagedRuntimeEventCredentialRequestV1) -> bool {
    value.request_id.len() == 16
        && value.request_id.iter().any(|byte| *byte != 0)
        && value.credential_revision > 0
        && (1..=600).contains(&value.ttl_seconds)
        && value.recipient_public_key_x25519.len() == 32
}

fn valid_provider_credential_request(value: &ManagedRuntimeProviderCredentialRequestV1) -> bool {
    value.request_id.len() == 16
        && value.request_id.iter().any(|byte| *byte != 0)
        && !value.purpose_id.trim().is_empty()
        && value.purpose_id.len() <= 128
        && value.credential_revision > 0
        && (1..=600).contains(&value.ttl_seconds)
        && (1..=5).contains(&value.secret_class)
        && (1..=6).contains(&value.action)
        && value.recipient_public_key_x25519.len() == 32
        && valid_configuration_instance_id(&value.configuration_instance_id)
}

fn valid_owner_derived_key_request(value: &ManagedRuntimeOwnerDerivedKeyRequestV1) -> bool {
    value.request_id.len() == 16
        && value.request_id.iter().any(|byte| *byte != 0)
        && !value.purpose_id.trim().is_empty()
        && value.purpose_id.len() <= 128
        && value.purpose_id.is_ascii()
        && value.key_schema_revision != 0
        && (1..=600).contains(&value.ttl_seconds)
        && value.recipient_public_key_x25519.len() == 32
}

fn valid_configuration_instance_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b'.')
        })
}
