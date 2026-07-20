//! File-backed source for first-boot platform credentials.

use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::credentials::PlatformCredentialSeeds;

const MAX_CREDENTIAL_BYTES: u64 = 65_536;
const PGBOUNCER_ADMIN_FILE: &str = "pgbouncer-admin-password";
const POSTGRES_ADMIN_FILE: &str = "postgres-admin-password";
const EVENT_HUB_NATS_FILE: &str = "nats-event-hub-password";
const EVENT_ACCOUNT_SIGNER_FILE: &str = "nats-account-signer-seed";

pub(super) struct FilePlatformCredentialSource {
    directory: PathBuf,
}

impl FilePlatformCredentialSource {
    pub(super) fn new(directory: &Path) -> Self {
        Self {
            directory: directory.to_owned(),
        }
    }

    pub(super) fn read(&self) -> Result<PlatformCredentialSeeds, String> {
        validate_directory(&self.directory)?;
        PlatformCredentialSeeds::new(
            read_credential(&self.directory.join(PGBOUNCER_ADMIN_FILE))?,
            read_credential(&self.directory.join(POSTGRES_ADMIN_FILE))?,
            read_optional_credential(&self.directory.join(EVENT_HUB_NATS_FILE))?,
            read_optional_credential(&self.directory.join(EVENT_ACCOUNT_SIGNER_FILE))?,
        )
    }
}

fn read_optional_credential(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => read_credential(path).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("Vault platform credential source is unavailable".to_owned()),
    }
}

fn validate_directory(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Vault platform credential source is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
    {
        return Err("Vault platform credential source is unavailable".to_owned());
    }
    Ok(())
}

fn read_credential(path: &Path) -> Result<Vec<u8>, String> {
    let before = std::fs::symlink_metadata(path)
        .map_err(|_| "Vault platform credential source is unavailable".to_owned())?;
    if before.file_type().is_symlink()
        || !before.is_file()
        || before.uid() != unsafe { libc::geteuid() }
        || before.mode() & 0o077 != 0
        || before.len() == 0
        || before.len() > MAX_CREDENTIAL_BYTES
    {
        return Err("Vault platform credential source is unavailable".to_owned());
    }
    let mut file = File::open(path)
        .map_err(|_| "Vault platform credential source is unavailable".to_owned())?;
    let opened = file
        .metadata()
        .map_err(|_| "Vault platform credential source is unavailable".to_owned())?;
    if !same_file(&before, &opened) {
        return Err("Vault platform credential source changed while it was opened".to_owned());
    }
    let mut value = Vec::with_capacity(
        usize::try_from(opened.len())
            .map_err(|_| "Vault platform credential source is unavailable".to_owned())?,
    );
    file.read_to_end(&mut value)
        .map_err(|_| "Vault platform credential source is unavailable".to_owned())?;
    let after = file
        .metadata()
        .and_then(|metadata| {
            std::fs::symlink_metadata(path).map(|path_metadata| (metadata, path_metadata))
        })
        .map_err(|_| "Vault platform credential source is unavailable".to_owned())?;
    if !same_file(&opened, &after.0) || !same_file(&opened, &after.1) {
        return Err("Vault platform credential source changed while it was read".to_owned());
    }
    while matches!(value.last(), Some(b'\n' | b'\r')) {
        value.pop();
    }
    (!value.is_empty())
        .then_some(value)
        .ok_or_else(|| "Vault platform credential source is unavailable".to_owned())
}

fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}
