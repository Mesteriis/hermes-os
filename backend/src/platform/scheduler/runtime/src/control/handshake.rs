//! Descriptor-bound authentication on the inherited Kernel FD.

use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use hermes_runtime_protocol::managed_control::ManagedControlChannelV2;
use hermes_runtime_protocol::v1::DescribeManagedRuntimeResponseV1;

pub(super) struct SchedulerRuntimeIdentity {
    registration_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

pub(super) fn authenticate(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, SchedulerRuntimeIdentity), String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Scheduler inherited control channel is unavailable".to_owned());
    }
    let stream = unsafe { UnixStream::from_raw_fd(duplicated) };
    authenticate_on_channel(stream, descriptor_bytes, settings_schema_bytes)
}

fn authenticate_on_channel(
    stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, SchedulerRuntimeIdentity), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| "Scheduler inherited control channel is unavailable".to_owned())?;
    let mut channel = ManagedControlChannelV2::new(stream);
    let identity = channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| "Scheduler managed-runtime descriptor was rejected".to_owned())?;
    Ok((
        channel.into_inner(),
        SchedulerRuntimeIdentity::from_describe(identity),
    ))
}

impl SchedulerRuntimeIdentity {
    fn from_describe(identity: DescribeManagedRuntimeResponseV1) -> Self {
        Self {
            registration_id: identity.registration_id,
            runtime_generation: identity.runtime_generation,
            grant_epoch: identity.grant_epoch,
        }
    }
    #[must_use]
    pub(super) fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub(super) const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub(super) const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}
