//! Ciphertext-only Vault route adapter for fenced Storage credentials.

mod context;
mod credentials;
mod platform_credential;
mod port;
mod session;

pub use context::{StorageVaultRouteContextErrorV1, StorageVaultRouteContextV1};
pub use credentials::StorageCredentialLeaseErrorV1;
pub use platform_credential::{
    StoragePlatformCredentialBootstrapV1, StoragePlatformCredentialErrorV1,
    StoragePlatformCredentialPurposeV1, StoragePlatformCredentialStateV1, complete_immediately,
};
pub use port::{StorageVaultRouteFailureV1, StorageVaultRoutePortV1};
pub use session::StorageVaultLeaseAdapterV1;
