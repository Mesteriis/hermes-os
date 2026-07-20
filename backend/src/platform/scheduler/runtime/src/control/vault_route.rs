//! Ciphertext-only Scheduler-to-Vault relay on the inherited managed channel.

use std::future::Future;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use hermes_runtime_protocol::v1::{
    ManagedRuntimeVaultRouteRequestV1, ManagedRuntimeVaultRouteResponseV1,
    VaultCiphertextResponseV1, VaultCiphertextRouteV1,
};
use hermes_storage_vault::{StorageVaultRouteFailureV1, StorageVaultRoutePortV1};
use prost::Message;

use super::framing::{read_frame, write_frame};

pub(super) struct InheritedSchedulerVaultRouteV1 {
    channel: UnixStream,
}

impl InheritedSchedulerVaultRouteV1 {
    pub(super) fn new(channel: UnixStream) -> Result<Self, ()> {
        channel
            .set_read_timeout(Some(Duration::from_secs(2)))
            .and_then(|_| channel.set_write_timeout(Some(Duration::from_secs(2))))
            .map_err(|_| ())?;
        Ok(Self { channel })
    }
}

impl StorageVaultRoutePortV1 for InheritedSchedulerVaultRouteV1 {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>> + Send
    {
        async move { route_once(&mut self.channel, route) }
    }
}

fn route_once(
    channel: &mut UnixStream,
    route: VaultCiphertextRouteV1,
) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
    write_frame(
        channel,
        &ManagedRuntimeVaultRouteRequestV1 { route: Some(route) }.encode_to_vec(),
    )
    .map_err(|_| StorageVaultRouteFailureV1::Unavailable)?;
    let response = ManagedRuntimeVaultRouteResponseV1::decode(
        read_frame(channel)
            .map_err(|_| StorageVaultRouteFailureV1::Unavailable)?
            .as_slice(),
    )
    .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    if !response.error_code.is_empty() {
        return Err(StorageVaultRouteFailureV1::Rejected);
    }
    response
        .response
        .ok_or(StorageVaultRouteFailureV1::Rejected)
}
