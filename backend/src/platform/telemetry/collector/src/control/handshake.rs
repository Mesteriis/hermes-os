//! One-shot descriptor handshake over Kernel's inherited control FD.

use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use hermes_runtime_protocol::managed_control::ManagedControlChannelV2;
use hermes_runtime_protocol::v1::ManagedRuntimeReadyRequestV1;

#[allow(dead_code)] // Used by the Collector inherited-channel composition harness.
pub fn describe(
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
) -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Telemetry inherited control is unavailable".to_owned());
    }
    let stream = unsafe { UnixStream::from_raw_fd(duplicated) };
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
        .map_err(|_| "Telemetry inherited control is unavailable".to_owned())?;
    let mut channel = ManagedControlChannelV2::new(stream);
    let identity = channel
        .describe_managed_runtime(descriptor_bytes, settings_schema_bytes)
        .map_err(|_| "Telemetry managed-runtime descriptor was rejected".to_owned())?;
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: identity.registration_id,
            runtime_generation: identity.runtime_generation,
            grant_epoch: identity.grant_epoch,
        })
        .map_err(|_| "Telemetry managed-runtime descriptor was rejected".to_owned())?;
    let stream = channel.into_inner();
    stream
        .set_read_timeout(None)
        .and_then(|_| stream.set_write_timeout(None))
        .map_err(|_| "Telemetry inherited control is unavailable".to_owned())?;
    Ok(stream)
}
