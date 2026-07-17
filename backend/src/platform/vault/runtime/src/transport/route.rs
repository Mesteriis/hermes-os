//! Private execution of an opaque, Kernel-fenced Vault ciphertext route.

use hermes_runtime_protocol::v1::{VaultCiphertextResponseV1, VaultCiphertextRouteV1};
use hermes_runtime_protocol::validation::vault::validate_vault_ciphertext_route_v1;
use hermes_vault_protocol::{
    LeaseAudienceV1, VaultCiphertextFrameV1, VaultTransportBindingV1, VaultTransportDirectionV1,
    VaultTransportSessionV1,
};
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use prost::Message;

use crate::service::runtime::VaultService;
use crate::transport::keys::VaultTransportKeyPair;
use crate::transport::response::encrypt_result;
use crate::transport::session::{VaultTransportReplayGuard, execute_session};

pub fn execute_route(
    service: &mut VaultService,
    keys: &VaultTransportKeyPair,
    replay_guard: &mut VaultTransportReplayGuard,
    authorization_key_sec1: [u8; 65],
    route: VaultCiphertextRouteV1,
    now_unix_seconds: u64,
) -> Result<VaultCiphertextResponseV1, String> {
    validate_vault_ciphertext_route_v1(&route)
        .map_err(|_| "Vault ciphertext route is invalid".to_owned())?;
    verify_kernel_authorization(&route, authorization_key_sec1)?;
    let binding = binding_from_route(&route)?;
    let frame = VaultCiphertextFrameV1::from_parts(
        route.hpke_encapped_key,
        route.ciphertext,
        route.hpke_authentication_tag,
    )
    .map_err(|_| "Vault ciphertext route is invalid".to_owned())?;
    let session = VaultTransportSessionV1::new(binding, frame);
    let plaintext = execute_session(replay_guard, keys, service, &session, now_unix_seconds)
        .map_err(|_| "Vault ciphertext route was denied".to_owned())?;
    encrypt_result(session.binding(), plaintext.as_slice())
}

pub(crate) fn verify_kernel_authorization(
    route: &VaultCiphertextRouteV1,
    key: [u8; 65],
) -> Result<(), String> {
    if route.kernel_instance_id.is_empty() || route.kernel_authorization_signature_raw.len() != 64 {
        return Err("Vault route authorization is invalid".to_owned());
    }
    let signature = Signature::from_slice(&route.kernel_authorization_signature_raw)
        .map_err(|_| "Vault route authorization is invalid".to_owned())?;
    let verifying_key = VerifyingKey::from_sec1_bytes(&key)
        .map_err(|_| "Vault route authorization is invalid".to_owned())?;
    let mut unsigned = route.clone();
    unsigned.kernel_authorization_signature_raw.clear();
    let mut message = b"hermes.vault-route-authorization.v1\0".to_vec();
    message.extend_from_slice(&unsigned.encode_to_vec());
    verifying_key
        .verify(&message, &signature)
        .map_err(|_| "Vault route authorization is invalid".to_owned())
}

fn binding_from_route(route: &VaultCiphertextRouteV1) -> Result<VaultTransportBindingV1, String> {
    let audience = LeaseAudienceV1::new(
        route.registration_id.clone(),
        route.runtime_instance_id.clone(),
        route.grant_epoch,
    )
    .map_err(|_| "Vault ciphertext route is invalid".to_owned())?;
    VaultTransportBindingV1::new(
        route.vault_runtime_generation,
        audience,
        fixed(&route.request_id)?,
        fixed(&route.operation_digest_sha256)?,
        VaultTransportDirectionV1::ToVault,
        fixed(&route.response_recipient_hpke_public_key_x25519)?,
    )
    .map_err(|_| "Vault ciphertext route is invalid".to_owned())
}

fn fixed<const N: usize>(value: &[u8]) -> Result<[u8; N], String> {
    value
        .try_into()
        .map_err(|_| "Vault ciphertext route is invalid".to_owned())
}
