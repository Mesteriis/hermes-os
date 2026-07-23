//! One-shot validation over the inherited private managed-runtime control FD.

use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::Stdio;
use std::sync::mpsc::SyncSender;
use std::time::Duration;

use hermes_kernel_control_store::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleRegistration,
    PlatformManagedProcessBinding, PlatformManagedProcessLaunch,
};
use hermes_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeEventCredentialDeliveryV1,
    ManagedRuntimeEventCredentialRequestV1, ManagedRuntimeProviderCredentialDeliveryV1,
    ManagedRuntimeProviderCredentialRequestV1, ManagedRuntimeBlobSessionDeliveryV1,
    ManagedRuntimeBlobSessionRequestV1, ManagedRuntimeVaultRouteRequestV1,
    VaultCiphertextResponseV1, VaultCiphertextRouteV1,
};
use hermes_runtime_protocol::validation::descriptor::{
    decode_descriptor_v1, decode_settings_schema_v1,
};
use hermes_runtime_protocol::validation::vault::validate_vault_ciphertext_route_v1;
use prost::Message;
use sha2::{Digest, Sha256};

const MAX_FRAME_BYTES: usize = 512 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

#[path = "control/inbound.rs"]
pub(crate) mod inbound;

pub trait ManagedRuntimeVaultRouteHandler: Send + Sync {
    fn route_vault_ciphertext(
        &self,
        expectation: &ManagedRuntimeExpectation,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, String>;
}

pub trait ManagedRuntimeEventCredentialHandler: Send + Sync {
    fn issue_event_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeEventCredentialRequestV1,
    ) -> Result<ManagedRuntimeEventCredentialDeliveryV1, String>;
}

pub trait ManagedRuntimeProviderCredentialHandler: Send + Sync {
    fn issue_provider_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeProviderCredentialRequestV1,
    ) -> Result<ManagedRuntimeProviderCredentialDeliveryV1, String>;
}

pub trait ManagedRuntimeBlobSessionHandler: Send + Sync {
    fn issue_blob_session(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobSessionRequestV1,
    ) -> Result<ManagedRuntimeBlobSessionDeliveryV1, String>;
}

pub struct ManagedRuntimeRelayRequest {
    payload: Vec<u8>,
    response: SyncSender<Result<Vec<u8>, String>>,
}

impl ManagedRuntimeRelayRequest {
    pub fn new(payload: Vec<u8>, response: SyncSender<Result<Vec<u8>, String>>) -> Self {
        Self { payload, response }
    }

    pub fn dispatch(
        self,
        channel: &mut UnixStream,
        expectation: &ManagedRuntimeExpectation,
        vault_route_handler: Option<&dyn ManagedRuntimeVaultRouteHandler>,
    ) {
        let _ = self.response.send(relay_with_vault_routes(
            channel,
            &self.payload,
            expectation,
            vault_route_handler,
        ));
    }
}

#[derive(Debug)]
pub struct ManagedRuntimeExpectation {
    registration_id: String,
    runtime_instance_id: String,
    module_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    descriptor_sha256: [u8; 32],
    settings_schema_sha256: Option<[u8; 32]>,
}

impl ManagedRuntimeExpectation {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        runtime_instance_id: impl Into<String>,
        module_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        descriptor_sha256: [u8; 32],
        settings_schema_sha256: Option<[u8; 32]>,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            runtime_instance_id: runtime_instance_id.into(),
            module_id: module_id.into(),
            runtime_generation,
            grant_epoch,
            descriptor_sha256,
            settings_schema_sha256,
        }
    }

    pub fn from_fenced_launch(
        registration: &ModuleRegistration,
        binding: &BundledManagedLaunchBinding,
        record: &ManagedLaunchRecord,
    ) -> Result<Self, String> {
        if registration.registration_id() != binding.registration_id()
            || registration.registration_id() != record.registration_id()
            || registration.descriptor_sha256() != binding.descriptor_sha256()
            || binding.binding_revision() != record.binding_revision()
            || registration.grant_epoch() != record.grant_epoch()
            || record.runtime_generation() == 0
        {
            return Err("managed launch fence does not match its approved registration".to_owned());
        }
        Ok(Self::new(
            registration.registration_id(),
            record.runtime_instance_id(),
            registration.module_id(),
            record.runtime_generation(),
            record.grant_epoch(),
            *binding.descriptor_sha256(),
            binding.settings_schema_sha256().copied(),
        ))
    }

    pub fn from_platform_fenced_launch(
        process_id: &str,
        module_id: &str,
        binding: &PlatformManagedProcessBinding,
        launch: &PlatformManagedProcessLaunch,
    ) -> Result<Self, String> {
        if process_id != binding.process_id()
            || process_id != launch.process_id()
            || binding.binding_revision() != launch.binding_revision()
            || launch.runtime_generation() == 0
        {
            return Err("platform managed launch fence does not match its binding".to_owned());
        }
        Ok(Self::new(
            process_id,
            process_id,
            module_id,
            launch.runtime_generation(),
            launch.grant_epoch(),
            *binding.descriptor_sha256(),
            binding.settings_schema_sha256().copied(),
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

    #[must_use]
    pub fn matches_ready(
        &self,
        ready: &hermes_runtime_protocol::v1::ManagedRuntimeReadyRequestV1,
    ) -> bool {
        ready.registration_id == self.registration_id
            && ready.runtime_generation == self.runtime_generation
            && ready.grant_epoch == self.grant_epoch
    }
}

pub fn create_inherited_channel() -> Result<(UnixStream, Stdio), String> {
    let (kernel_end, child_end) = UnixStream::pair().map_err(|error| error.to_string())?;
    let child_fd: OwnedFd = child_end.into();
    Ok((kernel_end, Stdio::from(child_fd)))
}

pub fn establish_channel(
    mut stream: UnixStream,
    expectation: &ManagedRuntimeExpectation,
) -> Result<UnixStream, String> {
    stream
        .set_read_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|error| error.to_string())?;
    let result = read_frame(&mut stream)
        .and_then(|bytes| {
            ManagedRuntimeControlRequestV1::decode(bytes.as_slice())
                .map_err(|_| "managed runtime control request is invalid".to_owned())
        })
        .and_then(|request| validate_describe(request, expectation));
    let response = match result {
        Ok(()) => ManagedRuntimeControlResponseV1 {
            result: Some(
                hermes_runtime_protocol::v1::managed_runtime_control_response_v1::Result::Describe(
                    DescribeManagedRuntimeResponseV1 {
                        registration_id: expectation.registration_id.clone(),
                        runtime_generation: expectation.runtime_generation,
                        grant_epoch: expectation.grant_epoch,
                    },
                ),
            ),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_runtime_describe_rejected".to_owned(),
        },
    };
    write_frame(&mut stream, &response.encode_to_vec())?;
    if result.is_ok() {
        stream
            .set_read_timeout(None)
            .and_then(|_| stream.set_write_timeout(None))
            .map_err(|error| error.to_string())?;
    }
    result.map(|()| stream)
}

pub fn relay(channel: &mut UnixStream, payload: &[u8]) -> Result<Vec<u8>, String> {
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err("managed runtime relay payload is invalid".to_owned());
    }
    write_frame(channel, payload)?;
    let response = read_frame(channel)?;
    if response.is_empty() {
        return Err("managed runtime relay response is invalid".to_owned());
    }
    Ok(response)
}

pub(crate) fn relay_with_vault_routes(
    channel: &mut UnixStream,
    payload: &[u8],
    expectation: &ManagedRuntimeExpectation,
    vault_route_handler: Option<&dyn ManagedRuntimeVaultRouteHandler>,
) -> Result<Vec<u8>, String> {
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err("managed runtime relay payload is invalid".to_owned());
    }
    write_frame(channel, payload)?;
    loop {
        let frame = read_frame(channel)?;
        let Some(route) = vault_route(&frame) else {
            return Ok(frame);
        };
        let result = vault_route_handler
            .ok_or_else(|| "managed runtime Vault route is not available".to_owned())?
            .route_vault_ciphertext(expectation, route);
        inbound::respond_vault_route(channel, result)?;
    }
}

fn vault_route(frame: &[u8]) -> Option<VaultCiphertextRouteV1> {
    let route = ManagedRuntimeVaultRouteRequestV1::decode(frame)
        .ok()?
        .route?;
    validate_vault_ciphertext_route_v1(&route).ok()?;
    Some(route)
}

fn validate_describe(
    request: ManagedRuntimeControlRequestV1,
    expectation: &ManagedRuntimeExpectation,
) -> Result<(), String> {
    use hermes_runtime_protocol::v1::managed_runtime_control_request_v1::Operation;

    let Some(Operation::Describe(describe)) = request.operation else {
        return Err("managed runtime control request is invalid".to_owned());
    };
    if Sha256::digest(&describe.descriptor_bytes).as_slice() != expectation.descriptor_sha256 {
        return Err("managed runtime descriptor digest does not match launch binding".to_owned());
    }
    let descriptor = decode_descriptor_v1(&describe.descriptor_bytes)
        .map_err(|_| "managed runtime descriptor is invalid".to_owned())?;
    if descriptor.module_id != expectation.module_id {
        return Err(
            "managed runtime descriptor module identity does not match launch binding".to_owned(),
        );
    }
    match expectation.settings_schema_sha256 {
        Some(expected_digest) => {
            if Sha256::digest(&describe.settings_schema_bytes).as_slice() != expected_digest {
                return Err(
                    "managed runtime settings schema digest does not match launch binding"
                        .to_owned(),
                );
            }
            decode_settings_schema_v1(&describe.settings_schema_bytes)
                .map_err(|_| "managed runtime settings schema is invalid".to_owned())?;
        }
        None if !describe.settings_schema_bytes.is_empty() => {
            return Err("managed runtime settings schema is not bound for this launch".to_owned());
        }
        None => {}
    }
    Ok(())
}

fn read_frame(stream: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = usize::try_from(read_varint(stream)?)
        .map_err(|_| "managed runtime control frame is too large".to_owned())?;
    if length > MAX_FRAME_BYTES {
        return Err("managed runtime control frame is too large".to_owned());
    }
    let mut bytes = vec![0_u8; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn read_varint(stream: &mut impl Read) -> Result<u64, String> {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("managed runtime control frame length is invalid".to_owned())
}

fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> Result<(), String> {
    let mut length = u32::try_from(bytes.len())
        .map_err(|_| "managed runtime control response is too large".to_owned())?;
    while length >= 0x80 {
        stream
            .write_all(&[(length as u8 & 0x7f) | 0x80])
            .map_err(|error| error.to_string())?;
        length >>= 7;
    }
    stream
        .write_all(&[length as u8])
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}
