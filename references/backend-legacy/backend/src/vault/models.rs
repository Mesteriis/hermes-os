use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::constants::MASTER_KEY_LEN;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostVaultConfig {
    pub home: PathBuf,
    pub dev_mode: bool,
    pub dev_key_path: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultMode {
    Uninitialized,
    Locked,
    Unlocked,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VaultStatus {
    pub state: VaultMode,
    pub needs_entropy: bool,
    pub needs_biometric: bool,
    pub needs_recovery: bool,
    pub version: u16,
    pub recoverable: bool,
    pub entropy_progress: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EntropyEvent {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub timestamp_ms: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub interval_ms: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct SecretEntryContext<'a> {
    pub entry_kind: &'a str,
    pub account_id: &'a str,
    pub purpose: &'a str,
    pub secret_kind: &'a str,
    pub label: &'a str,
    pub metadata: &'a serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveryExportResponse {
    pub path: PathBuf,
    pub recovery_phrase: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HostVaultManifestEntry {
    pub secret_ref: String,
    pub entry_kind: String,
    pub account_id: String,
    pub purpose: String,
    pub secret_kind: String,
    pub store_kind: String,
    pub label: String,
    pub metadata: serde_json::Value,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct RecoveryFile {
    pub(super) version: u16,
    pub(super) nonce: String,
    pub(super) ciphertext: String,
}

#[derive(Debug)]
pub(super) struct StoredVaultEntry {
    pub(super) version: u16,
    pub(super) nonce: String,
    pub(super) ciphertext: String,
    pub(super) aad: String,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub(super) struct SessionKey {
    pub(super) bytes: [u8; MASTER_KEY_LEN],
}

impl SessionKey {
    pub(super) fn new(bytes: [u8; MASTER_KEY_LEN]) -> Self {
        Self { bytes }
    }
}

pub(super) enum HostVaultState {
    Locked,
    Unlocked(SessionKey),
}
