//! Fenced provider and Storage credential resolution through the inherited Kernel route.

use std::future::Future;
use std::os::unix::net::UnixStream;

use hermes_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use hermes_storage_protocol::StorageBindingV1;
use hermes_storage_vault::{
    StorageCredentialLeaseErrorV1, StorageVaultLeaseAdapterV1,
    StorageVaultRouteContextV1, StorageVaultRouteFailureV1, StorageVaultRoutePortV1,
};
use hermes_vault_protocol::{
    seal, CredentialLeaseV1, LeaseAudienceV1, SecretClassV1, VaultCiphertextFrameV1,
    VaultResponseRecipientV1, VaultTransportBindingV1, VaultTransportCommandV1,
    VaultTransportDirectionV1, VaultTransportPublicKey,
};
use zeroize::Zeroizing;

use crate::managed_control::route_vault_ciphertext;

const REQUEST_ID_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WhatsAppVaultRouteContext {
    pub vault_runtime_generation: u64,
    pub vault_public_key: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppCredentialRouteError {
    InvalidContext,
    InvalidLease,
    Unavailable,
    Rejected,
}

pub fn resolve_credential_lease(
    channel: &mut UnixStream,
    context: WhatsAppVaultRouteContext,
    lease: &CredentialLeaseV1,
) -> Result<Zeroizing<Vec<u8>>, WhatsAppCredentialRouteError> {
    if context.vault_runtime_generation == 0 {
        return Err(WhatsAppCredentialRouteError::InvalidContext);
    }
    let vault_public_key = VaultTransportPublicKey::from_bytes(context.vault_public_key)
        .map_err(|_| WhatsAppCredentialRouteError::InvalidContext)?;
    let request = lease.request();
    if request.vault_runtime_generation() != context.vault_runtime_generation {
        return Err(WhatsAppCredentialRouteError::InvalidLease);
    }
    let audience = request.audience().clone();
    let command = VaultTransportCommandV1::ResolveLease {
        lease_id: lease.lease_id().clone(),
        secret_class: SecretClassV1::SessionCredentialBlob,
    };
    let request_id = random_request_id().ok_or(WhatsAppCredentialRouteError::Rejected)?;
    let recipient = VaultResponseRecipientV1::generate();
    let request_binding = transport_binding(&audience, context.vault_runtime_generation, request_id, &command, &recipient, VaultTransportDirectionV1::ToVault)?;
    let response_binding = transport_binding(&audience, context.vault_runtime_generation, request_id, &command, &recipient, VaultTransportDirectionV1::FromVault)?;
    let frame = seal(&vault_public_key, &request_binding, &command.encode())
        .map_err(|_| WhatsAppCredentialRouteError::Rejected)?;
    let route = VaultCiphertextRouteV1 {
        major: 1,
        registration_id: audience.module_registration_id().to_owned(),
        runtime_instance_id: audience.runtime_instance_id().to_owned(),
        vault_runtime_generation: context.vault_runtime_generation,
        grant_epoch: audience.grant_epoch(),
        request_id: request_id.to_vec(),
        operation_digest_sha256: command.operation_digest().to_vec(),
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: frame.encapped_key().to_vec(),
        ciphertext: frame.ciphertext().to_vec(),
        hpke_authentication_tag: frame.tag().to_vec(),
        response_recipient_hpke_public_key_x25519: recipient.public_key().as_bytes().to_vec(),
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        caller_runtime_generation: audience.runtime_generation(),
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    };
    let response = route_vault_ciphertext(channel, route.clone())
        .map_err(|_| WhatsAppCredentialRouteError::Unavailable)?;
    let response_frame = validated_response(&route, response)
        .ok_or(WhatsAppCredentialRouteError::Rejected)?;
    recipient
        .open(&response_binding, &response_frame)
        .map_err(|_| WhatsAppCredentialRouteError::Rejected)
}

fn transport_binding(
    audience: &LeaseAudienceV1,
    vault_runtime_generation: u64,
    request_id: [u8; REQUEST_ID_BYTES],
    command: &VaultTransportCommandV1,
    recipient: &VaultResponseRecipientV1,
    direction: VaultTransportDirectionV1,
) -> Result<VaultTransportBindingV1, WhatsAppCredentialRouteError> {
    VaultTransportBindingV1::new(
        vault_runtime_generation,
        audience.clone(),
        request_id,
        command.operation_digest(),
        direction,
        *recipient.public_key().as_bytes(),
    )
    .map_err(|_| WhatsAppCredentialRouteError::Rejected)
}

fn validated_response(
    request: &VaultCiphertextRouteV1,
    response: VaultCiphertextResponseV1,
) -> Option<VaultCiphertextFrameV1> {
    if response.major != 1
        || response.vault_runtime_generation != request.vault_runtime_generation
        || response.caller_runtime_generation != request.caller_runtime_generation
        || response.request_id != request.request_id
        || response.operation_digest_sha256 != request.operation_digest_sha256
        || response.direction != VaultCiphertextRouteDirectionV1::FromVault as i32
    {
        return None;
    }
    VaultCiphertextFrameV1::from_parts(
        response.hpke_encapped_key,
        response.ciphertext,
        response.hpke_authentication_tag,
    )
    .ok()
}

fn random_request_id() -> Result<[u8; REQUEST_ID_BYTES], ()> {
    let mut request_id = [0; REQUEST_ID_BYTES];
    getrandom::fill(&mut request_id).map_err(|_| ())?;
    Ok(request_id)
}

struct InheritedWhatsAppVaultRoute {
    channel: UnixStream,
}

impl InheritedWhatsAppVaultRoute {
    fn new(channel: UnixStream) -> Self {
        Self { channel }
    }
}

impl StorageVaultRoutePortV1 for InheritedWhatsAppVaultRoute {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<
        Output = Result<hermes_runtime_protocol::v1::VaultCiphertextResponseV1, StorageVaultRouteFailureV1>,
    > + Send {
        async move {
            route_vault_ciphertext(&mut self.channel, route)
                .map_err(|_| StorageVaultRouteFailureV1::Unavailable)
        }
    }
}

pub async fn resolve_storage_credential(
    channel: UnixStream,
    binding: &StorageBindingV1,
    context: StorageVaultRouteContextV1,
) -> Result<Zeroizing<Vec<u8>>, StorageCredentialLeaseErrorV1> {
    let mut leases = StorageVaultLeaseAdapterV1::new(
        InheritedWhatsAppVaultRoute::new(channel),
        context,
    );
    let lease_id = leases.issue_runtime_credential(binding).await?;
    leases.resolve_runtime_credential(binding, lease_id).await
}
