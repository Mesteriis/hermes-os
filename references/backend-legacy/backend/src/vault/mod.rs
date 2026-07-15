mod constants;
mod crypto;
pub mod errors;
mod files;
mod key_store;
mod lifecycle;
mod manifest;
pub mod models;
pub mod paths;
mod recovery;
mod secrets;
mod storage;

use models::EntropyEvent;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use models::HostVaultState;

#[derive(Clone)]
pub struct HostVault {
    home: PathBuf,
    dev_mode: bool,
    dev_key_path: PathBuf,
    state: Arc<Mutex<HostVaultState>>,
    entropy: Arc<Mutex<Vec<EntropyEvent>>>,
}
