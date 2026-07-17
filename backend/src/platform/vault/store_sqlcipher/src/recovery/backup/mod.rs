//! Offline, encrypted Vault snapshot export. This is not whole-instance backup.

mod api;
mod database;
mod manifest;
mod paths;

pub(crate) use database::{export_database_snapshot, validate_database};
pub use manifest::{VaultBackupClassV1, VaultBackupManifestV1};
pub(crate) use manifest::{create_manifest, verify_manifest};
pub(crate) use paths::{create_destination, open_destination, sync_directory};
