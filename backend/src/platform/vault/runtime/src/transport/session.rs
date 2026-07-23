//! Vault-private authentication and replay fencing for HPKE transport sessions.

use std::collections::BTreeSet;

use hermes_runtime_protocol::v1::VaultCiphertextRouteV1;
use hermes_vault_protocol::{
    SecretClassV1, VaultActionV1, VaultTransportBindingV1, VaultTransportCommandV1,
    VaultTransportDirectionV1, VaultTransportError, VaultTransportSessionV1,
};
use zeroize::Zeroizing;

use crate::service::runtime::{VaultService, VaultServiceError};
use crate::transport::keys::VaultTransportKeyPair;

pub const MAX_TRANSPORT_SESSIONS: usize = 1_024;

pub struct VaultTransportReplayGuard {
    runtime_generation: u64,
    consumed_request_ids: BTreeSet<[u8; 16]>,
}

impl VaultTransportReplayGuard {
    #[must_use]
    pub fn new(runtime_generation: u64) -> Self {
        Self {
            runtime_generation,
            consumed_request_ids: BTreeSet::new(),
        }
    }

    pub fn open_command(
        &mut self,
        keys: &VaultTransportKeyPair,
        session: &VaultTransportSessionV1,
    ) -> Result<VaultTransportCommandV1, VaultTransportError> {
        self.validate_binding(session)?;
        let plaintext = keys.open(session.binding(), session.frame())?;
        let command = VaultTransportCommandV1::decode(&plaintext)
            .map_err(|_| VaultTransportError::InvalidBinding)?;
        if command.operation_digest() != *session.binding().operation_digest() {
            return Err(VaultTransportError::InvalidBinding);
        }
        self.consume_request_id(*session.binding().request_id())?;
        Ok(command)
    }

    fn validate_binding(
        &self,
        session: &VaultTransportSessionV1,
    ) -> Result<(), VaultTransportError> {
        if session.binding().vault_runtime_generation() != self.runtime_generation {
            return Err(VaultTransportError::InvalidBinding);
        }
        if session.binding().direction() != VaultTransportDirectionV1::ToVault {
            return Err(VaultTransportError::WrongDirection);
        }
        Ok(())
    }

    fn consume_request_id(&mut self, request_id: [u8; 16]) -> Result<(), VaultTransportError> {
        if self.consumed_request_ids.contains(&request_id) {
            return Err(VaultTransportError::ReplayDetected);
        }
        if self.consumed_request_ids.len() == MAX_TRANSPORT_SESSIONS {
            return Err(VaultTransportError::SessionCapacityExceeded);
        }
        self.consumed_request_ids.insert(request_id);
        Ok(())
    }
}

pub fn execute_session(
    guard: &mut VaultTransportReplayGuard,
    keys: &VaultTransportKeyPair,
    service: &mut VaultService,
    session: &VaultTransportSessionV1,
    now_unix_seconds: u64,
) -> Result<Zeroizing<Vec<u8>>, VaultSessionExecutionError> {
    let command = guard
        .open_command(keys, session)
        .map_err(VaultSessionExecutionError::Transport)?;
    service
        .execute_command_once(&command, session.binding().audience(), now_unix_seconds)
        .map_err(VaultSessionExecutionError::Service)
}

pub fn execute_storage_session(
    guard: &mut VaultTransportReplayGuard,
    keys: &VaultTransportKeyPair,
    service: &mut VaultService,
    session: &VaultTransportSessionV1,
    route: &VaultCiphertextRouteV1,
    now_unix_seconds: u64,
) -> Result<Zeroizing<Vec<u8>>, VaultSessionExecutionError> {
    let command = guard
        .open_command(keys, session)
        .map_err(VaultSessionExecutionError::Transport)?;
    validate_storage_command(&command, session.binding(), route)
        .map_err(VaultSessionExecutionError::Transport)?;
    service
        .execute_command_once(&command, session.binding().audience(), now_unix_seconds)
        .map_err(VaultSessionExecutionError::Service)
}

fn validate_storage_command(
    command: &VaultTransportCommandV1,
    binding: &VaultTransportBindingV1,
    route: &VaultCiphertextRouteV1,
) -> Result<(), VaultTransportError> {
    match command {
        VaultTransportCommandV1::IssueLease { request } => {
            let purpose = request.purpose();
            (request.vault_runtime_generation() == binding.vault_runtime_generation()
                && request.secret_revision() == route.storage_credential_lease_revision
                && request.logical_owner_id() == route.storage_owner_id
                && request.audience() == binding.audience()
                && purpose.purpose_id() == "storage.runtime.credential"
                && purpose.configuration_instance_id() == route.storage_runtime_principal
                && purpose.allowed_secret_classes() == [SecretClassV1::PlatformCredential]
                && matches!(
                    purpose.actions(),
                    [VaultActionV1::Create] | [VaultActionV1::Resolve]
                ))
            .then_some(())
            .ok_or(VaultTransportError::InvalidBinding)
        }
        VaultTransportCommandV1::ResolveLease { secret_class, .. }
        | VaultTransportCommandV1::GenerateOpaqueToken { secret_class, .. } => (*secret_class
            == SecretClassV1::PlatformCredential)
            .then_some(())
            .ok_or(VaultTransportError::InvalidBinding),
        VaultTransportCommandV1::RevokeAudience => Ok(()),
        VaultTransportCommandV1::StoreLease { .. }
        | VaultTransportCommandV1::EnsureOwnerDerivedKey { .. }
        | VaultTransportCommandV1::ReplaceLease { .. } => Err(VaultTransportError::InvalidBinding),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum VaultSessionExecutionError {
    Transport(VaultTransportError),
    Service(VaultServiceError),
}
