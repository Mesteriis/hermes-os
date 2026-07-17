//! Crash-recovery reservation journal for offline root-key rotation.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use getrandom::fill;
use sha2::{Digest, Sha256};

use super::paths;
use crate::database::store::VaultStoreError;

const MAGIC: &[u8; 8] = b"HVRROT01";
const BYTES: usize = 72;

pub(super) struct Reservation {
    database_digest: [u8; 32],
    anchor_digest: [u8; 32],
}

impl Reservation {
    pub(super) fn from_staged(database: &Path, anchor: &Path) -> Result<Self, VaultStoreError> {
        Ok(Self {
            database_digest: digest(database)?,
            anchor_digest: digest(anchor)?,
        })
    }

    pub(super) fn database_digest(&self) -> [u8; 32] {
        self.database_digest
    }

    pub(super) fn anchor_digest(&self) -> [u8; 32] {
        self.anchor_digest
    }

    fn encode(&self) -> [u8; BYTES] {
        let mut bytes = [0; BYTES];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..40].copy_from_slice(&self.database_digest);
        bytes[40..72].copy_from_slice(&self.anchor_digest);
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, VaultStoreError> {
        if bytes.len() != BYTES || &bytes[..8] != MAGIC {
            return Err(VaultStoreError::RootRotationPending);
        }
        Ok(Self {
            database_digest: bytes[8..40]
                .try_into()
                .map_err(|_| VaultStoreError::RootRotationPending)?,
            anchor_digest: bytes[40..72]
                .try_into()
                .map_err(|_| VaultStoreError::RootRotationPending)?,
        })
    }
}

pub(super) fn exists(database_path: &Path) -> Result<bool, VaultStoreError> {
    match std::fs::symlink_metadata(paths::journal_path(database_path)?) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(VaultStoreError::InsecurePath),
    }
}

pub(super) fn write(
    database_path: &Path,
    reservation: &Reservation,
) -> Result<(), VaultStoreError> {
    let journal = paths::journal_path(database_path)?;
    let temporary = temporary_path(&journal)?;
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|_| VaultStoreError::InsecurePath)?;
        file.write_all(&reservation.encode())
            .and_then(|_| file.sync_all())
            .map_err(|_| VaultStoreError::InsecurePath)?;
        std::fs::rename(&temporary, &journal).map_err(|_| VaultStoreError::InsecurePath)?;
        paths::sync_parent(&journal)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

pub(super) fn read(database_path: &Path) -> Result<Reservation, VaultStoreError> {
    Reservation::decode(&read_private_regular_file(&paths::journal_path(
        database_path,
    )?)?)
}

pub(super) fn remove(database_path: &Path) -> Result<(), VaultStoreError> {
    let journal = paths::journal_path(database_path)?;
    validate_private_regular_file(&journal)?;
    std::fs::remove_file(journal).map_err(|_| VaultStoreError::InsecurePath)?;
    paths::sync_parent(database_path)
}

pub(super) fn digest(path: &Path) -> Result<[u8; 32], VaultStoreError> {
    Ok(Sha256::digest(read_private_regular_file(path)?).into())
}

fn read_private_regular_file(path: &Path) -> Result<Vec<u8>, VaultStoreError> {
    let before =
        std::fs::symlink_metadata(path).map_err(|_| VaultStoreError::RootRotationPending)?;
    if !is_private_regular_file(&before) {
        return Err(VaultStoreError::RootRotationPending);
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| VaultStoreError::RootRotationPending)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| VaultStoreError::RootRotationPending)?;
    let opened = file
        .metadata()
        .map_err(|_| VaultStoreError::RootRotationPending)?;
    let path_after =
        std::fs::symlink_metadata(path).map_err(|_| VaultStoreError::RootRotationPending)?;
    if same_file(&before, &opened) && same_file(&before, &path_after) {
        Ok(bytes)
    } else {
        Err(VaultStoreError::RootRotationPending)
    }
}

fn validate_private_regular_file(path: &Path) -> Result<(), VaultStoreError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| VaultStoreError::InsecurePath)?;
    is_private_regular_file(&metadata)
        .then_some(())
        .ok_or(VaultStoreError::InsecurePath)
}

fn is_private_regular_file(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == unsafe { libc::geteuid() }
        && metadata.mode() & 0o077 == 0
}

fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
}

fn temporary_path(path: &Path) -> Result<std::path::PathBuf, VaultStoreError> {
    let parent = path.parent().ok_or(VaultStoreError::InsecurePath)?;
    let name = path.file_name().ok_or(VaultStoreError::InsecurePath)?;
    let mut suffix = [0_u8; 16];
    fill(&mut suffix).map_err(|_| VaultStoreError::InsecurePath)?;
    let suffix = suffix
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(parent.join(format!(".{}.{}.tmp", name.to_string_lossy(), suffix)))
}
