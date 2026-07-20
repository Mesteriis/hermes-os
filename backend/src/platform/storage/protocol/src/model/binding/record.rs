//! Complete StorageBindingV1 assembled from independently validated parts.

use super::{
    StorageBindingAccessV1, StorageBindingErrorV1, StorageBindingFencesV1, StorageBindingIdentityV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageBindingV1 {
    identity: StorageBindingIdentityV1,
    fences: StorageBindingFencesV1,
    access: StorageBindingAccessV1,
}

impl StorageBindingV1 {
    pub fn new(
        identity: StorageBindingIdentityV1,
        fences: StorageBindingFencesV1,
        access: StorageBindingAccessV1,
    ) -> Result<Self, StorageBindingErrorV1> {
        validate_pool_alias(&identity, fences, &access)?;
        Ok(Self {
            identity,
            fences,
            access,
        })
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

fn validate_pool_alias(
    identity: &StorageBindingIdentityV1,
    fences: StorageBindingFencesV1,
    access: &StorageBindingAccessV1,
) -> Result<(), StorageBindingErrorV1> {
    let expected = format!(
        "runtime_{}_{}",
        identity.registration_id(),
        fences.runtime_generation()
    );
    if access.pool_alias() != expected || !valid_pool_alias(&expected) {
        return Err(StorageBindingErrorV1::PoolAlias);
    }
    Ok(())
}

fn valid_pool_alias(value: &str) -> bool {
    value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
