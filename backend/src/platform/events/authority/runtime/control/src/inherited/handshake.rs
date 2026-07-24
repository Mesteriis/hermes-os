//! Descriptor-bound identity handshake on the inherited Kernel FD.

use std::os::unix::net::UnixStream;
use std::time::Duration;

use hermes_runtime_protocol::managed_control::ManagedControlChannelV2;
use hermes_runtime_protocol::v1::DescribeManagedRuntimeResponseV1;

pub(crate) struct EventsAuthorityRuntimeIdentityV1 {
    registration_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

pub(crate) fn authenticate(
    stream: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<(UnixStream, EventsAuthorityRuntimeIdentityV1), String> {
    let stream = stream;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| "Events authority inherited channel is unavailable".to_owned())?;
    let mut channel = ManagedControlChannelV2::new(stream);
    let identity = channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| "Events authority managed-runtime descriptor was rejected".to_owned())?;
    if !valid_id(&identity.registration_id) {
        return Err("Events authority managed-runtime descriptor was rejected".to_owned());
    }
    Ok((
        channel.into_inner(),
        EventsAuthorityRuntimeIdentityV1::from_describe(identity),
    ))
}

impl EventsAuthorityRuntimeIdentityV1 {
    fn from_describe(identity: DescribeManagedRuntimeResponseV1) -> Self {
        Self {
            registration_id: identity.registration_id,
            runtime_generation: identity.runtime_generation,
            grant_epoch: identity.grant_epoch,
        }
    }
    pub(crate) fn registration_id(&self) -> &str {
        &self.registration_id
    }
    pub(crate) const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
    pub(crate) const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
