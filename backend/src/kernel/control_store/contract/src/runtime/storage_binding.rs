//! Durable non-secret fences and lifecycle state for one Storage binding.

use super::PlatformStorageBindingStateV1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformStorageBindingV1 {
    registration_id: String,
    capability_id: String,
    owner_id: String,
    binding_revision: u64,
    topology_revision: u64,
    storage_generation: u64,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    role_epoch: u64,
    runtime_principal: String,
    connection_budget: u16,
    statement_timeout_millis: u32,
    credential_lease_revision: u64,
    storage_bundle_revision: u64,
    storage_bundle_digest: [u8; 32],
    state: PlatformStorageBindingStateV1,
}

struct BindingFields {
    registration_id: String,
    capability_id: String,
    owner_id: String,
    binding_revision: u64,
    topology_revision: u64,
    storage_generation: u64,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    role_epoch: u64,
    runtime_principal: String,
    connection_budget: u16,
    statement_timeout_millis: u32,
    credential_lease_revision: u64,
    storage_bundle_revision: u64,
    storage_bundle_digest: [u8; 32],
}

impl BindingFields {
    fn is_valid(&self) -> bool {
        valid_identifier(&self.registration_id)
            && valid_capability_id(&self.capability_id)
            && valid_owner_id(&self.owner_id)
            && valid_identifier(&self.runtime_instance_id)
            && valid_identifier(&self.runtime_principal)
            && self.connection_budget > 0
            && self.statement_timeout_millis > 0
            && [
                self.binding_revision,
                self.topology_revision,
                self.storage_generation,
                self.runtime_generation,
                self.grant_epoch,
                self.role_epoch,
                self.credential_lease_revision,
                self.storage_bundle_revision,
            ]
            .iter()
            .all(|value| *value > 0)
            && self.storage_bundle_digest.iter().any(|value| *value != 0)
    }

    fn into_active(self) -> PlatformStorageBindingV1 {
        PlatformStorageBindingV1 {
            registration_id: self.registration_id,
            capability_id: self.capability_id,
            owner_id: self.owner_id,
            binding_revision: self.binding_revision,
            topology_revision: self.topology_revision,
            storage_generation: self.storage_generation,
            runtime_instance_id: self.runtime_instance_id,
            runtime_generation: self.runtime_generation,
            grant_epoch: self.grant_epoch,
            role_epoch: self.role_epoch,
            runtime_principal: self.runtime_principal,
            connection_budget: self.connection_budget,
            statement_timeout_millis: self.statement_timeout_millis,
            credential_lease_revision: self.credential_lease_revision,
            storage_bundle_revision: self.storage_bundle_revision,
            storage_bundle_digest: self.storage_bundle_digest,
            state: PlatformStorageBindingStateV1::Active,
        }
    }
}

impl PlatformStorageBindingV1 {
    pub fn new(
        registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        owner_id: impl Into<String>,
        binding_revision: u64,
        topology_revision: u64,
        storage_generation: u64,
        runtime_instance_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        role_epoch: u64,
        runtime_principal: impl Into<String>,
        connection_budget: u16,
        statement_timeout_millis: u32,
        credential_lease_revision: u64,
        storage_bundle_revision: u64,
        storage_bundle_digest: [u8; 32],
    ) -> Result<Self, ()> {
        let fields = BindingFields {
            registration_id: registration_id.into(),
            capability_id: capability_id.into(),
            owner_id: owner_id.into(),
            binding_revision,
            topology_revision,
            storage_generation,
            runtime_instance_id: runtime_instance_id.into(),
            runtime_generation,
            grant_epoch,
            role_epoch,
            runtime_principal: runtime_principal.into(),
            connection_budget,
            statement_timeout_millis,
            credential_lease_revision,
            storage_bundle_revision,
            storage_bundle_digest,
        };
        fields.is_valid().then(|| fields.into_active()).ok_or(())
    }

    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    pub const fn binding_revision(&self) -> u64 {
        self.binding_revision
    }
    pub const fn topology_revision(&self) -> u64 {
        self.topology_revision
    }
    pub const fn storage_generation(&self) -> u64 {
        self.storage_generation
    }
    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
    }
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
    pub const fn role_epoch(&self) -> u64 {
        self.role_epoch
    }
    pub fn runtime_principal(&self) -> &str {
        &self.runtime_principal
    }
    pub const fn connection_budget(&self) -> u16 {
        self.connection_budget
    }
    pub const fn statement_timeout_millis(&self) -> u32 {
        self.statement_timeout_millis
    }
    pub const fn credential_lease_revision(&self) -> u64 {
        self.credential_lease_revision
    }
    pub const fn storage_bundle_revision(&self) -> u64 {
        self.storage_bundle_revision
    }
    pub const fn storage_bundle_digest(&self) -> &[u8; 32] {
        &self.storage_bundle_digest
    }
    pub const fn state(&self) -> PlatformStorageBindingStateV1 {
        self.state
    }

    pub fn begin_revocation(&self) -> Result<Self, ()> {
        (self.state == PlatformStorageBindingStateV1::Active)
            .then(|| Self {
                state: PlatformStorageBindingStateV1::Revoking,
                ..self.clone()
            })
            .ok_or(())
    }

    pub fn restore_state(mut self, state: PlatformStorageBindingStateV1) -> Self {
        self.state = state;
        self
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_capability_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn valid_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
