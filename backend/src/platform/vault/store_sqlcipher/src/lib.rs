//! SQLCipher persistence boundary for Vault state.

mod actor;
mod database;
mod identity;
mod records;
mod recovery;

pub use database::store::{VaultStore, VaultStoreError};
pub use identity::{VaultRecoveryKeyError, VaultRecoveryKeyV1};
pub use records::secret::{SecretRecordId, SecretRecordScope};
pub use recovery::backup::{VaultBackupClassV1, VaultBackupManifestV1};
