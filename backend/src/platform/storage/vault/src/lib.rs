//! Shared Storage-owned encrypted Vault credential route.

mod route;

pub use route::{
    InheritedKernelVaultRouteV1,
    KernelCredentialLeaseErrorV1, KernelVaultLeaseContextV1,
    StorageCredentialLeaseErrorV1, StoragePlatformCredentialBootstrapV1,
    StoragePlatformCredentialErrorV1, StoragePlatformCredentialPurposeV1,
    StoragePlatformCredentialStateV1, StorageVaultLeaseAdapterV1, StorageVaultRouteContextErrorV1,
    StorageVaultRouteContextV1, StorageVaultRouteFailureV1, StorageVaultRoutePortV1,
    complete_immediately, resolve_kernel_credential_lease,
};
