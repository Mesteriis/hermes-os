//! Monotonic generations and epochs that fence a storage runtime.

use super::StorageBindingErrorV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBindingFencesV1 {
    storage_generation: u64,
    runtime_generation: u64,
    grant_epoch: u64,
    role_epoch: u64,
    credential_lease_revision: u64,
    storage_bundle_revision: u64,
}

impl StorageBindingFencesV1 {
    pub fn new(
        storage_generation: u64,
        runtime_generation: u64,
        grant_epoch: u64,
        role_epoch: u64,
        credential_lease_revision: u64,
        storage_bundle_revision: u64,
    ) -> Result<Self, StorageBindingErrorV1> {
        if [
            storage_generation,
            runtime_generation,
            grant_epoch,
            role_epoch,
            credential_lease_revision,
            storage_bundle_revision,
        ]
        .contains(&0)
        {
            return Err(StorageBindingErrorV1::Fence);
        }
        Ok(Self {
            storage_generation,
            runtime_generation,
            grant_epoch,
            role_epoch,
            credential_lease_revision,
            storage_bundle_revision,
        })
    }

    pub const fn storage_generation(&self) -> u64 {
        self.storage_generation
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

    pub const fn credential_lease_revision(&self) -> u64 {
        self.credential_lease_revision
    }

    pub const fn storage_bundle_revision(&self) -> u64 {
        self.storage_bundle_revision
    }
}
