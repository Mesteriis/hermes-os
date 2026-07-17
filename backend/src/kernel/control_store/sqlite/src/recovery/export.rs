//! Crash-safe Control Store export through the online SQLite actor.

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, backup::Backup};
use sha2::{Digest, Sha256};

use crate::database::connection::configure_writable;
use crate::{SqliteControlStore, StoreError};

pub struct ControlStoreExport {
    instance_id: String,
    generation: u64,
    sha256: [u8; 32],
    sha256_hex: String,
}

impl ControlStoreExport {
    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }
    #[must_use]
    pub fn sha256_hex(&self) -> &str {
        &self.sha256_hex
    }
    #[must_use]
    pub fn sha256_bytes(&self) -> &[u8; 32] {
        &self.sha256
    }
}

impl SqliteControlStore {
    pub fn export_to(&self, destination: &Path) -> Result<ControlStoreExport, StoreError> {
        if destination == self.path {
            return Err(StoreError::InvalidExportDestination);
        }
        let parent = destination
            .parent()
            .ok_or(StoreError::InvalidExportDestination)?;
        let temporary = create_export_temporary_file(parent, destination)?;
        let actor_temporary = temporary.clone();
        let export_result =
            self.with_maintenance_connection(move |source| write_export(source, &actor_temporary));
        if let Err(error) = export_result {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        install_export(&temporary, destination, parent)?;

        let digest: [u8; 32] = Sha256::digest(std::fs::read(destination)?).into();
        Ok(ControlStoreExport {
            instance_id: self.snapshot().instance_id().to_owned(),
            generation: self.snapshot().generation(),
            sha256: digest,
            sha256_hex: digest.iter().map(|byte| format!("{byte:02x}")).collect(),
        })
    }
}

fn write_export(source: &Connection, destination: &Path) -> Result<(), StoreError> {
    let mut output = Connection::open(destination)?;
    configure_writable(&output)?;
    let backup = Backup::new(source, &mut output)?;
    backup.run_to_completion(32, Duration::from_millis(10), None)?;
    drop(backup);
    output.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    Ok(())
}

fn install_export(temporary: &Path, destination: &Path, parent: &Path) -> Result<(), StoreError> {
    File::open(temporary)?.sync_all()?;
    std::fs::rename(temporary, destination)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn create_export_temporary_file(parent: &Path, destination: &Path) -> Result<PathBuf, StoreError> {
    for attempt in 0..16 {
        let name = destination
            .file_name()
            .ok_or(StoreError::InvalidExportDestination)?
            .to_string_lossy();
        let temporary = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), attempt));
        match open_new_private_file(&temporary) {
            Ok(()) => return Ok(temporary),
            Err(StoreError::Io(error)) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(StoreError::InvalidExportDestination)
}

fn open_new_private_file(path: &Path) -> Result<(), StoreError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    Ok(())
}
