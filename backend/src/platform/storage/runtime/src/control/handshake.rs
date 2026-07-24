//! Descriptor-bound handshake over the Kernel-provided inherited FD.

use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use hermes_runtime_protocol::managed_control::ManagedControlChannelV2;
use hermes_vault_protocol::LeaseAudienceV1;

// Test composition exercises the descriptor handshake through the same
// inherited-channel path as the managed runtime. Production uses `authenticate`.
#[allow(dead_code)]
pub fn describe(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Storage inherited control channel is unavailable".to_owned());
    }
    let stream = unsafe { UnixStream::from_raw_fd(duplicated) };
    authenticate_on_channel(stream, descriptor_bytes, settings_schema_bytes)
        .map(|(channel, _)| channel)
}

#[allow(dead_code)]
pub fn describe_on_channel(
    stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<UnixStream, String> {
    authenticate_on_channel(stream, descriptor_bytes, settings_schema_bytes)
        .map(|(channel, _)| channel)
}

pub(super) fn authenticate(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, ManagedStorageRuntimeIdentityV1), String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Storage inherited control channel is unavailable".to_owned());
    }
    let stream = unsafe { UnixStream::from_raw_fd(duplicated) };
    authenticate_on_channel(stream, descriptor_bytes, settings_schema_bytes)
}

pub(super) fn authenticate_on_channel(
    stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, ManagedStorageRuntimeIdentityV1), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| "Storage inherited control channel is unavailable".to_owned())?;
    let mut channel = ManagedControlChannelV2::new(stream);
    let identity = channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| "Storage managed-runtime descriptor was rejected".to_owned())?;
    let audience = LeaseAudienceV1::new(
        identity.registration_id,
        "storage-runtime".to_owned(),
        identity.runtime_generation,
        identity.grant_epoch,
    )
    .map_err(|_| "Storage managed-runtime descriptor was rejected".to_owned())?;
    Ok((
        channel.into_inner(),
        ManagedStorageRuntimeIdentityV1 { audience },
    ))
}

#[derive(Clone)]
pub(super) struct ManagedStorageRuntimeIdentityV1 {
    audience: LeaseAudienceV1,
}

impl ManagedStorageRuntimeIdentityV1 {
    #[must_use]
    pub(super) const fn runtime_generation(&self) -> u64 {
        self.audience.runtime_generation()
    }

    #[must_use]
    pub(super) fn audience(&self) -> LeaseAudienceV1 {
        self.audience.clone()
    }
}
