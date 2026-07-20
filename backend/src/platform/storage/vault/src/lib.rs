//! Shared Storage-owned encrypted Vault credential route.

mod route;

pub use route::{
    StorageCredentialLeaseErrorV1, StoragePlatformCredentialBootstrapV1,
    StoragePlatformCredentialErrorV1, StoragePlatformCredentialPurposeV1,
    StoragePlatformCredentialStateV1, StorageVaultLeaseAdapterV1, StorageVaultRouteContextErrorV1,
    StorageVaultRouteContextV1, StorageVaultRouteFailureV1, StorageVaultRoutePortV1,
    complete_immediately,
};
