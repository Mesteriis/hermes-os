//! Vault lease and secret-resolution core, independent from IPC transport.

use hermes_vault_protocol::{
    CredentialLeaseV1, LeaseAudienceV1, LeaseIdV1, VaultActionV1, VaultLeaseIssueRequestV1,
    VaultTransportCommandV1,
};
use hermes_vault_store_sqlcipher::{SecretRecordId, SecretRecordScope, VaultStore};
use zeroize::Zeroizing;

use crate::service::leases::{LeaseError, LeaseManager};

pub struct VaultService {
    store: VaultStore,
    leases: LeaseManager,
}

impl VaultService {
    pub fn new(store: VaultStore, runtime_generation: u64) -> Result<Self, VaultServiceError> {
        let leases = LeaseManager::new(store.instance_id().to_owned(), runtime_generation)
            .map_err(VaultServiceError::Lease)?;
        Ok(Self { store, leases })
    }

    pub fn issue_lease(
        &mut self,
        request: VaultLeaseIssueRequestV1,
        now_unix_seconds: u64,
    ) -> Result<CredentialLeaseV1, VaultServiceError> {
        if request
            .purpose()
            .actions()
            .iter()
            .any(|action| !supported_action(*action))
        {
            return Err(VaultServiceError::UnsupportedLeaseAction);
        }
        self.leases
            .issue(request, now_unix_seconds)
            .map_err(VaultServiceError::Lease)
    }

    pub fn resolve_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        record_id: &SecretRecordId,
        scope: &SecretRecordScope,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(lease_id, audience, VaultActionV1::Resolve, now_unix_seconds)?;
        if !scope.matches_lease_request(lease.request()) {
            return Err(VaultServiceError::LeaseScopeMismatch);
        }
        self.store
            .resolve_scoped_secret(record_id, scope)
            .map_err(|_| VaultServiceError::SecretUnavailable)
    }

    pub fn replace_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        prior_record_id: &SecretRecordId,
        prior_scope: &SecretRecordScope,
        next_scope: &SecretRecordScope,
        payload: &[u8],
        now_unix_seconds: u64,
    ) -> Result<SecretRecordId, VaultServiceError> {
        let lease = self.consume_action(
            lease_id,
            audience,
            VaultActionV1::ReplaceCas,
            now_unix_seconds,
        )?;
        if !next_scope.matches_lease_request(lease.request()) {
            return Err(VaultServiceError::LeaseScopeMismatch);
        }
        self.store
            .replace_secret(prior_record_id, prior_scope, next_scope, payload)
            .map_err(|_| VaultServiceError::SecretUnavailable)
    }

    pub fn execute_command_once(
        &mut self,
        command: &VaultTransportCommandV1,
        audience: &LeaseAudienceV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        match command {
            VaultTransportCommandV1::ResolveLease {
                lease_id,
                secret_class,
            } => self.resolve_current_once(lease_id, audience, *secret_class, now_unix_seconds),
            VaultTransportCommandV1::StoreLease {
                lease_id,
                secret_class,
                payload,
            } => self.store_current_once(
                lease_id,
                audience,
                *secret_class,
                payload,
                now_unix_seconds,
            ),
            VaultTransportCommandV1::ReplaceLease {
                lease_id,
                secret_class,
                prior_record_id,
                payload,
            } => self.replace_current_once(
                lease_id,
                audience,
                *secret_class,
                prior_record_id,
                payload,
                now_unix_seconds,
            ),
        }
    }

    pub fn revoke_audience(&mut self, audience: &LeaseAudienceV1) {
        self.leases.invalidate_audience(audience);
    }

    pub fn advance_runtime_generation(
        &mut self,
        next_generation: u64,
    ) -> Result<(), VaultServiceError> {
        self.leases
            .advance_generation(next_generation)
            .map_err(VaultServiceError::Lease)
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.leases.runtime_generation()
    }

    fn consume_action(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        action: VaultActionV1,
        now_unix_seconds: u64,
    ) -> Result<CredentialLeaseV1, VaultServiceError> {
        let lease = self
            .leases
            .consume_once(lease_id, audience, now_unix_seconds)
            .map_err(VaultServiceError::Lease)?;
        if lease.request().purpose().actions().contains(&action) {
            Ok(lease)
        } else {
            Err(VaultServiceError::LeaseActionDenied)
        }
    }

    fn resolve_current_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: hermes_vault_protocol::SecretClassV1,
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(lease_id, audience, VaultActionV1::Resolve, now_unix_seconds)?;
        let scope = SecretRecordScope::new(
            lease.request().logical_owner_id().to_owned(),
            lease.request().purpose(),
            secret_class,
            lease.request().secret_revision(),
        )
        .map_err(|_| VaultServiceError::LeaseScopeMismatch)?;
        self.store
            .resolve_current_secret(&scope)
            .map_err(|_| VaultServiceError::SecretUnavailable)
    }

    fn store_current_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: hermes_vault_protocol::SecretClassV1,
        payload: &[u8],
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease =
            self.consume_action(lease_id, audience, VaultActionV1::Create, now_unix_seconds)?;
        let scope = SecretRecordScope::new(
            lease.request().logical_owner_id().to_owned(),
            lease.request().purpose(),
            secret_class,
            lease.request().secret_revision(),
        )
        .map_err(|_| VaultServiceError::LeaseScopeMismatch)?;
        let record_id = self
            .store
            .store_secret(&scope, payload)
            .map_err(|_| VaultServiceError::SecretUnavailable)?;
        Ok(Zeroizing::new(record_id.as_bytes().to_vec()))
    }

    fn replace_current_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        secret_class: hermes_vault_protocol::SecretClassV1,
        prior_record_id: &[u8; 16],
        payload: &[u8],
        now_unix_seconds: u64,
    ) -> Result<Zeroizing<Vec<u8>>, VaultServiceError> {
        let lease = self.consume_action(
            lease_id,
            audience,
            VaultActionV1::ReplaceCas,
            now_unix_seconds,
        )?;
        let next_scope = scope_for_lease(&lease, secret_class, lease.request().secret_revision())?;
        let prior_revision = lease
            .request()
            .secret_revision()
            .checked_sub(1)
            .filter(|revision| *revision > 0)
            .ok_or(VaultServiceError::LeaseScopeMismatch)?;
        let prior_scope = scope_for_lease(&lease, secret_class, prior_revision)?;
        let record_id = self
            .store
            .replace_secret(
                &SecretRecordId::from_bytes(*prior_record_id),
                &prior_scope,
                &next_scope,
                payload,
            )
            .map_err(|_| VaultServiceError::SecretUnavailable)?;
        Ok(Zeroizing::new(record_id.as_bytes().to_vec()))
    }
}

fn scope_for_lease(
    lease: &CredentialLeaseV1,
    secret_class: hermes_vault_protocol::SecretClassV1,
    revision: u64,
) -> Result<SecretRecordScope, VaultServiceError> {
    SecretRecordScope::new(
        lease.request().logical_owner_id().to_owned(),
        lease.request().purpose(),
        secret_class,
        revision,
    )
    .map_err(|_| VaultServiceError::LeaseScopeMismatch)
}

fn supported_action(action: VaultActionV1) -> bool {
    matches!(
        action,
        VaultActionV1::Resolve | VaultActionV1::Create | VaultActionV1::ReplaceCas
    )
}

#[derive(Debug, Eq, PartialEq)]
pub enum VaultServiceError {
    Lease(LeaseError),
    UnsupportedLeaseAction,
    LeaseActionDenied,
    LeaseScopeMismatch,
    SecretUnavailable,
}
