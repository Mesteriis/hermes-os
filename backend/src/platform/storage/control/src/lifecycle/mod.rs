//! Binding lifecycle transitions before infrastructure side effects.

mod error;
mod preflight;
mod provisioning;
mod revocation;
mod state;

pub use error::StorageLifecycleErrorV1;
pub use preflight::{StorageEndpointPreflightV1, preflight_storage_endpoints};
pub use provisioning::{
    StorageProvisionerV1, StorageProvisioningErrorV1, StorageProvisioningFailureV1,
    StorageProvisioningPortV1,
};
pub use revocation::{
    StorageFenceOutcomeV1, StoragePoolFenceCommandV1, StoragePoolFencePortV1,
    StoragePostgresFencePortV1, StorageRevocationErrorV1, StorageRevocationReportV1,
    StorageRevokerV1, StorageVaultLeasePortV1,
};
pub use state::{StorageLifecycleStateV1, StorageLifecycleV1};
