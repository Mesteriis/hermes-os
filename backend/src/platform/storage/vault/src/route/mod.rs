//! Ciphertext-only Vault route adapter for fenced Storage credentials.

mod context;
mod credentials;
mod inherited_kernel;
mod kernel_credential;
mod platform_credential;
mod port;
mod session;

pub use context::{StorageVaultRouteContextErrorV1, StorageVaultRouteContextV1};
pub use credentials::StorageCredentialLeaseErrorV1;
pub use inherited_kernel::InheritedKernelVaultRouteV1;
pub use kernel_credential::{
    KernelCredentialLeaseErrorV1, KernelVaultLeaseContextV1, resolve_kernel_credential_lease,
};
pub use platform_credential::{
    StoragePlatformCredentialBootstrapV1, StoragePlatformCredentialErrorV1,
    StoragePlatformCredentialPurposeV1, StoragePlatformCredentialStateV1, complete_immediately,
};
pub use port::{StorageVaultRouteFailureV1, StorageVaultRoutePortV1};
pub use session::StorageVaultLeaseAdapterV1;
