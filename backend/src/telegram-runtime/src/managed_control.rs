//! Telegram managed-runtime channel: descriptor handshake and opaque Vault routing.

use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use hermes_runtime_protocol::v1::{
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedRuntimeVaultRouteRequestV1, ManagedRuntimeVaultRouteResponseV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

const MAX_FRAME_BYTES: usize = 512 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramManagedRuntimeIdentity {
    registration_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl TelegramManagedRuntimeIdentity {
    pub fn open_inherited(
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        runtime_instance_id: impl Into<String>,
    ) -> Result<(Self, UnixStream), String> {
        let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
        if duplicated < 0 {
            return Err("Telegram managed-runtime channel is unavailable".to_owned());
        }
        let channel = unsafe { UnixStream::from_raw_fd(duplicated as RawFd) };
        Self::authenticate(channel, descriptor_bytes, settings_schema_bytes, runtime_instance_id)
    }

    pub fn authenticate(
        mut channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        runtime_instance_id: impl Into<String>,
    ) -> Result<(Self, UnixStream), String> {
        if descriptor_bytes.is_empty() || settings_schema_bytes.is_empty() {
            return Err("Telegram managed-runtime descriptor is empty".to_owned());
        }
        let runtime_instance_id = runtime_instance_id.into();
        if runtime_instance_id.trim().is_empty() {
            return Err("Telegram managed-runtime instance id is empty".to_owned());
        }
        channel
            .set_read_timeout(Some(CONTROL_TIMEOUT))
            .and_then(|_| channel.set_write_timeout(Some(CONTROL_TIMEOUT)))
            .map_err(|_| "Telegram managed-runtime channel is unavailable".to_owned())?;
        write_frame(
            &mut channel,
            &ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes,
                    settings_schema_bytes,
                })),
            }
            .encode_to_vec(),
        )?;
        let response = ManagedRuntimeControlResponseV1::decode(read_frame(&mut channel)?.as_slice())
            .map_err(|_| "Telegram managed-runtime describe response is invalid".to_owned())?;
        let (registration_id, runtime_generation, grant_epoch) = match response.result {
            Some(ControlResult::Describe(value))
                if response.error_code.is_empty()
                    && !value.registration_id.is_empty()
                    && value.runtime_generation != 0
                    && value.grant_epoch != 0 =>
            {
                (value.registration_id, value.runtime_generation, value.grant_epoch)
            }
            _ => return Err("Telegram managed-runtime descriptor was rejected".to_owned()),
        };
        write_frame(
            &mut channel,
            &ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1 {
                    registration_id: registration_id.clone(),
                    runtime_generation,
                    grant_epoch,
                })),
            }
            .encode_to_vec(),
        )?;
        channel
            .set_read_timeout(None)
            .and_then(|_| channel.set_write_timeout(None))
            .map_err(|_| "Telegram managed-runtime channel is unavailable".to_owned())?;
        Ok((
            Self {
                registration_id,
                runtime_instance_id,
                runtime_generation,
                grant_epoch,
            },
            channel,
        ))
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

pub fn route_vault_ciphertext(
    channel: &mut UnixStream,
    route: hermes_runtime_protocol::v1::VaultCiphertextRouteV1,
) -> Result<hermes_runtime_protocol::v1::VaultCiphertextResponseV1, String> {
    write_frame(
        channel,
        &ManagedRuntimeVaultRouteRequestV1 { route: Some(route) }.encode_to_vec(),
    )?;
    let response = ManagedRuntimeVaultRouteResponseV1::decode(read_frame(channel)?.as_slice())
        .map_err(|_| "Telegram managed-runtime Vault response is invalid".to_owned())?;
    response
        .response
        .filter(|_| response.error_code.is_empty())
        .ok_or_else(|| "Telegram managed-runtime Vault route was denied".to_owned())
}

fn read_frame(channel: &mut UnixStream) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(channel)?)
        .map_err(|_| "Telegram managed-runtime frame is invalid".to_owned())?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err("Telegram managed-runtime frame is invalid".to_owned());
    }
    let mut bytes = vec![0_u8; length];
    channel
        .read_exact(&mut bytes)
        .map_err(|_| "Telegram managed-runtime channel is unavailable".to_owned())?;
    Ok(bytes)
}

fn write_frame(channel: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err("Telegram managed-runtime frame is invalid".to_owned());
    }
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "Telegram managed-runtime frame is invalid".to_owned())?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    channel
        .write_all(&prefix)
        .and_then(|_| channel.write_all(bytes))
        .and_then(|_| channel.flush())
        .map_err(|_| "Telegram managed-runtime channel is unavailable".to_owned())
}

fn read_varint(channel: &mut UnixStream) -> Result<u64, String> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel
            .read_exact(&mut byte)
            .map_err(|_| "Telegram managed-runtime channel is unavailable".to_owned())?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("Telegram managed-runtime frame is invalid".to_owned())
}
