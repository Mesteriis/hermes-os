//! Concrete ciphertext-only Vault relay over the inherited managed channel.

use std::future::Future;
use std::os::unix::net::UnixStream;

use hermes_runtime_protocol::v1::{
    ManagedRuntimeVaultRouteRequestV1, ManagedRuntimeVaultRouteResponseV1,
    VaultCiphertextResponseV1, VaultCiphertextRouteV1,
};
use prost::Message;

use crate::vault::{StorageVaultRouteFailureV1, StorageVaultRoutePortV1};

use super::framing::{read_frame, write_frame};

pub struct InheritedVaultRoutePortV1 {
    channel: UnixStream,
}

impl InheritedVaultRoutePortV1 {
    #[must_use]
    pub fn new(channel: UnixStream) -> Self {
        Self { channel }
    }
}

impl StorageVaultRoutePortV1 for InheritedVaultRoutePortV1 {
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
    let request = ManagedRuntimeVaultRouteRequestV1 { route: Some(route) };
    write_frame(channel, &request.encode_to_vec())
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
