//! Persistence port for durable Storage binding fences.

use crate::PlatformStorageBindingV1;

pub trait StorageBindingStore {
    type Error;

    fn record_platform_storage_binding(
        &self,
        binding: &PlatformStorageBindingV1,
    ) -> Result<(), Self::Error>;
    fn platform_storage_binding(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<PlatformStorageBindingV1>, Self::Error>;
    fn begin_platform_storage_binding_revocation(
        &self,
        registration_id: &str,
        capability_id: &str,
        binding_revision: u64,
    ) -> Result<PlatformStorageBindingV1, Self::Error>;
    fn platform_storage_bindings(&self) -> Result<Vec<PlatformStorageBindingV1>, Self::Error>;
}
