//! Complete StorageBindingV1 assembled from independently validated parts.

use super::{StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageBindingV1 {
    identity: StorageBindingIdentityV1,
    fences: StorageBindingFencesV1,
    access: StorageBindingAccessV1,
}

impl StorageBindingV1 {
    pub const fn new(
        identity: StorageBindingIdentityV1,
        fences: StorageBindingFencesV1,
        access: StorageBindingAccessV1,
    ) -> Self {
        Self {
            identity,
            fences,
            access,
        }
    }

    pub const fn identity(&self) -> &StorageBindingIdentityV1 {
        &self.identity
    }

    pub const fn fences(&self) -> StorageBindingFencesV1 {
        self.fences
    }

    pub fn access(&self) -> &StorageBindingAccessV1 {
        &self.access
    }
}
