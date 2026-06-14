mod crypto;
mod database_vault;
mod errors;
mod file_vault;
mod models;
mod paths;
mod resolver;
mod store;
mod validation;

pub use database_vault::{DatabaseEncryptedSecretVault, DatabaseEncryptedVaultError};
pub use errors::{SecretReferenceError, SecretResolutionError};
pub use file_vault::{EncryptedSecretVault, EncryptedVaultError};
pub use models::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReference, SecretStoreKind,
};
pub use paths::default_vault_path;
pub use resolver::{InMemorySecretResolver, SecretResolutionFuture, SecretResolver};
pub use store::SecretReferenceStore;
