//! Persistence port for canonical owner-local Storage migration bundles.

use crate::PlatformStorageBundleV1;

pub trait StorageBundleStore {
    type Error;

    fn record_platform_storage_bundle(
        &self,
        bundle: &PlatformStorageBundleV1,
    ) -> Result<(), Self::Error>;
    fn platform_storage_bundle(
        &self,
        owner_id: &str,
        revision: u64,
    ) -> Result<Option<PlatformStorageBundleV1>, Self::Error>;
}
