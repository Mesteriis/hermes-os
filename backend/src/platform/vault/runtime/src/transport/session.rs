//! Vault-private authentication and replay fencing for HPKE transport sessions.

use std::collections::BTreeSet;

use hermes_vault_protocol::{
    VaultTransportCommandV1, VaultTransportDirectionV1, VaultTransportError,
    VaultTransportSessionV1,
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

#[derive(Debug, Eq, PartialEq)]
pub enum VaultSessionExecutionError {
    Transport(VaultTransportError),
    Service(VaultServiceError),
}
