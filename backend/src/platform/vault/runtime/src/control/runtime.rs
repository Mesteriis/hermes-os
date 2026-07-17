//! Vault runtime loop served exclusively over the inherited Kernel channel.

use prost::Message;

use hermes_runtime_protocol::v1::VaultCiphertextRouteV1;

use crate::control::inherited::{open_and_describe, read_frame, write_frame};
use crate::service::runtime::VaultService;
use crate::transport::keys::VaultTransportKeyPair;
use crate::transport::route::execute_route;
use crate::transport::session::VaultTransportReplayGuard;

pub fn serve(
    service: &mut VaultService,
    keys: &VaultTransportKeyPair,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    authorization_key_sec1: [u8; 65],
) -> Result<(), String> {
    let mut channel = open_and_describe(descriptor_bytes, settings_schema_bytes)?;
    let mut replay_guard = VaultTransportReplayGuard::new(service.runtime_generation());
    loop {
        let route = VaultCiphertextRouteV1::decode(read_frame(&mut channel)?.as_slice())
            .map_err(|_| "Vault inherited control frame is invalid".to_owned())?;
        let response = execute_route(
            service,
            keys,
            &mut replay_guard,
            authorization_key_sec1,
            route,
            unix_seconds()?,
        )?;
        write_frame(&mut channel, &response.encode_to_vec())?;
    }
}

fn unix_seconds() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "Vault clock is unavailable".to_owned())
}
