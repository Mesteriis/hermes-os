//! Ciphertext-only Blob-to-Vault relay on the inherited managed channel.

use std::future::Future;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use hermes_blob_runtime::vault::{BlobVaultRouteFailureV1, BlobVaultRoutePortV1};
use hermes_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeVaultRouteRequestV1, VaultCiphertextResponseV1, VaultCiphertextRouteV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use super::framing::{read_frame, write_frame};

pub(super) struct InheritedBlobVaultRouteV1 {
    channel: UnixStream,
}

impl InheritedBlobVaultRouteV1 {
    pub(super) fn new(channel: UnixStream) -> Result<Self, ()> {
        channel
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| ())?;
        channel
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| ())?;
        Ok(Self { channel })
    }
}

impl BlobVaultRoutePortV1 for InheritedBlobVaultRouteV1 {
    #[allow(clippy::manual_async_fn)] // The Blob-to-Vault port requires a Send future.
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, BlobVaultRouteFailureV1>> + Send
    {
        async move { route_once(&mut self.channel, route) }
    }
}

fn route_once(
    channel: &mut UnixStream,
    route: VaultCiphertextRouteV1,
) -> Result<VaultCiphertextResponseV1, BlobVaultRouteFailureV1> {
    write_frame(
        channel,
        &ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::RouteVaultCiphertext(
                ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
            )),
        }
        .encode_to_vec(),
    )
    .map_err(|_| BlobVaultRouteFailureV1::Unavailable)?;
    let response = ManagedRuntimeControlResponseV1::decode(
        read_frame(channel)
            .map_err(|_| BlobVaultRouteFailureV1::Unavailable)?
            .as_slice(),
    )
    .map_err(|_| BlobVaultRouteFailureV1::Rejected)?
    .result
    .and_then(|result| match result {
        ControlResult::VaultRoute(response) => Some(response),
        _ => None,
    })
    .ok_or(BlobVaultRouteFailureV1::Rejected)?;
    if !response.error_code.is_empty() {
        return Err(BlobVaultRouteFailureV1::Rejected);
    }
    response.response.ok_or(BlobVaultRouteFailureV1::Rejected)
}
