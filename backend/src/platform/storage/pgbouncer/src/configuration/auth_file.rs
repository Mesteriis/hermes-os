//! Private writer for the PgBouncer authentication file.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use zeroize::Zeroizing;

use super::PoolConfigErrorV1;

const PRIVATE_MODE: u32 = 0o600;
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

pub struct PgBouncerAuthFileV1 {
    path: PathBuf,
}

pub struct PgBouncerAuthEntryV1 {
    username: String,
    secret: Zeroizing<String>,
}

impl PgBouncerAuthEntryV1 {
    pub fn pooler_admin(
        username: &str,
        password: &Zeroizing<Vec<u8>>,
    ) -> Result<Self, PoolConfigErrorV1> {
        let password = std::str::from_utf8(password).map_err(|_| PoolConfigErrorV1::Identifier)?;
        Self::new(username, password, false)
    }

    pub fn runtime_scram(
        username: &str,
        verifier: Zeroizing<String>,
    ) -> Result<Self, PoolConfigErrorV1> {
        if !valid_scram_verifier(&verifier) {
            return Err(PoolConfigErrorV1::Identifier);
        }
        Self::new_zeroizing(username, verifier)
    }

    fn new(username: &str, secret: &str, scram_only: bool) -> Result<Self, PoolConfigErrorV1> {
        if scram_only && !valid_scram_verifier(secret) {
            return Err(PoolConfigErrorV1::Identifier);
        }
        Self::new_zeroizing(username, Zeroizing::new(secret.to_owned()))
    }

    fn new_zeroizing(username: &str, secret: Zeroizing<String>) -> Result<Self, PoolConfigErrorV1> {
        (valid_identifier(username) && valid_secret(&secret))
            .then_some(Self {
                username: username.to_owned(),
                secret,
            })
            .ok_or(PoolConfigErrorV1::Identifier)
    }
}

impl PgBouncerAuthFileV1 {
    pub fn replace(
        path: &Path,
        mut entries: Vec<PgBouncerAuthEntryV1>,
    ) -> Result<Self, PoolConfigErrorV1> {
        validate_path(path)?;
        entries.sort_by(|left, right| left.username.cmp(&right.username));
        if entries
            .windows(2)
            .any(|pair| pair[0].username == pair[1].username)
        {
            return Err(PoolConfigErrorV1::Identifier);
        }
        let contents = render_entries(entries);
        write_replacement(path, contents.as_bytes())?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn render_entries(entries: Vec<PgBouncerAuthEntryV1>) -> Zeroizing<String> {
    let mut output = Zeroizing::new(String::new());
    for entry in entries {
        output.push('"');
        output.push_str(&entry.username);
        output.push_str("\" \"");
        output.push_str(&entry.secret);
        output.push_str("\"\n");
    }
    output
}

fn validate_path(path: &Path) -> Result<(), PoolConfigErrorV1> {
    let parent = path.parent().ok_or(PoolConfigErrorV1::FileSystem)?;
    let metadata = std::fs::symlink_metadata(parent).map_err(|_| PoolConfigErrorV1::FileSystem)?;
    (metadata.is_dir()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == current_uid()
        && metadata.mode() & 0o777 == PRIVATE_DIRECTORY_MODE)
        .then_some(())
        .ok_or(PoolConfigErrorV1::FileSystem)?;
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if trusted_file(&metadata) => Ok(()),
        Ok(_) => Err(PoolConfigErrorV1::FileSystem),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(PoolConfigErrorV1::FileSystem),
    }
}

fn write_replacement(path: &Path, contents: &[u8]) -> Result<(), PoolConfigErrorV1> {
    if path.exists() {
        return write_existing(path, contents);
    }
    let temporary = temporary_path(path)?;
    let result = write_temporary(&temporary, contents).and_then(|()| replace(path, &temporary));
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn write_existing(path: &Path, contents: &[u8]) -> Result<(), PoolConfigErrorV1> {
    // Keep the existing inode: Docker Desktop bind mounts can transiently expose
    // an atomic rename as a missing auth_file to the PgBouncer container.
    // PgBouncer reads this file only after the caller has fully written it and
    // explicitly issued RELOAD.
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| PoolConfigErrorV1::FileSystem)?;
    trusted_file(&file.metadata().map_err(|_| PoolConfigErrorV1::FileSystem)?)
        .then_some(())
        .ok_or(PoolConfigErrorV1::FileSystem)?;
    file.write_all(contents)
        .and_then(|()| file.sync_all())
        .map_err(|_| PoolConfigErrorV1::FileSystem)
}

fn temporary_path(path: &Path) -> Result<PathBuf, PoolConfigErrorV1> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| path.with_file_name(format!(".{name}.next")))
        .ok_or(PoolConfigErrorV1::FileSystem)
}

fn write_temporary(path: &Path, contents: &[u8]) -> Result<(), PoolConfigErrorV1> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_MODE)
        .open(path)
        .map_err(|_| PoolConfigErrorV1::FileSystem)?;
    file.write_all(contents)
        .and_then(|()| file.sync_all())
        .map_err(|_| PoolConfigErrorV1::FileSystem)?;
    trusted_file(&file.metadata().map_err(|_| PoolConfigErrorV1::FileSystem)?)
        .then_some(())
        .ok_or(PoolConfigErrorV1::FileSystem)
}

fn replace(path: &Path, temporary: &Path) -> Result<(), PoolConfigErrorV1> {
    std::fs::rename(temporary, path).map_err(|_| PoolConfigErrorV1::FileSystem)?;
    let parent = path.parent().ok_or(PoolConfigErrorV1::FileSystem)?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| PoolConfigErrorV1::FileSystem)
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_secret(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4 * 1024
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'"' | b'\\'))
}

fn valid_scram_verifier(value: &str) -> bool {
    value.starts_with("SCRAM-SHA-256$") && valid_secret(value)
}

fn trusted_file(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == current_uid()
        && metadata.mode() & 0o777 == PRIVATE_MODE
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and returns the effective UID only.
    unsafe { libc::geteuid() }
}
