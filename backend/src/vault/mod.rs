mod constants;
mod crypto;
mod errors;
mod files;
mod key_store;
mod lifecycle;
mod manifest;
mod models;
mod paths;
mod provider_accounts;
mod recovery;
mod secrets;
mod storage;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub use errors::HostVaultError;
use models::HostVaultState;
pub use models::{
    EntropyEvent, HostVaultConfig, HostVaultManifestEntry, RecoveryExportResponse,
    SecretEntryContext, VaultMode, VaultStatus,
};
pub use paths::{default_dev_key_path, default_vault_home};
pub use provider_accounts::{
    CalendarAccountStore, CalendarSourceStore, CommunicationProviderAccountStore,
    CommunicationProviderSecretBindingStore, TaskProviderStore,
};

#[derive(Clone)]
pub struct HostVault {
    home: PathBuf,
    dev_mode: bool,
    dev_key_path: PathBuf,
    state: Arc<Mutex<HostVaultState>>,
    entropy: Arc<Mutex<Vec<EntropyEvent>>>,
}
