//! Ciphertext-only Vault relay over the verified inherited control channel.

use std::future::Future;
use std::os::unix::net::UnixStream;

use hermes_events_jetstream::{NatsVaultRouteFailureV1, NatsVaultRoutePortV1};
use hermes_runtime_protocol::v1::{
    ManagedRuntimeVaultRouteRequestV1, ManagedRuntimeVaultRouteResponseV1,
    VaultCiphertextResponseV1, VaultCiphertextRouteV1,
};
use prost::Message;

use super::framing::{read_frame, write_frame};

pub(crate) struct InheritedVaultRoutePortV1 {
    channel: UnixStream,
}

impl InheritedVaultRoutePortV1 {
    pub(crate) fn new(channel: UnixStream) -> Self {
        Self { channel }
    }
}

impl NatsVaultRoutePortV1 for InheritedVaultRoutePortV1 {
    #[allow(clippy::manual_async_fn)] // The NATS-to-Vault port requires a Send future.
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, NatsVaultRouteFailureV1>> + Send
    {
        async move { route_once(&mut self.channel, route) }
    }
}

fn route_once(
    channel: &mut UnixStream,
    route: VaultCiphertextRouteV1,
) -> Result<VaultCiphertextResponseV1, NatsVaultRouteFailureV1> {
    let request = ManagedRuntimeVaultRouteRequestV1 { route: Some(route) };
    write_frame(channel, &request.encode_to_vec())
        .map_err(|_| NatsVaultRouteFailureV1::Unavailable)?;
    let response = ManagedRuntimeVaultRouteResponseV1::decode(
        read_frame(channel)
            .map_err(|_| NatsVaultRouteFailureV1::Unavailable)?
            .as_slice(),
    )
    .map_err(|_| NatsVaultRouteFailureV1::Rejected)?;
    if !response.error_code.is_empty() {
        return Err(NatsVaultRouteFailureV1::Rejected);
    }
    response.response.ok_or(NatsVaultRouteFailureV1::Rejected)
}
