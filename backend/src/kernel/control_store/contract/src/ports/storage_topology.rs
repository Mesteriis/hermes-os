use crate::PlatformStorageTopology;

/// Kernel-only desired topology for the Storage platform stack.
pub trait StorageTopologyStore {
    type Error;

    fn record_platform_storage_topology(
        &self,
        topology: &PlatformStorageTopology,
    ) -> Result<(), Self::Error>;
    fn platform_storage_topology(&self) -> Result<Option<PlatformStorageTopology>, Self::Error>;
}
