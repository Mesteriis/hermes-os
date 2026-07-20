//! Ownership-checked writer for Storage-owned PgBouncer database entries.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use super::{PgBouncerRuntimeConfigV1, PoolConfigErrorV1};

const PRIVATE_MODE: u32 = 0o600;
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

pub struct PgBouncerDatabaseConfigFileV1 {
    path: PathBuf,
}

impl PgBouncerDatabaseConfigFileV1 {
    pub fn replace(
        path: &Path,
        entries: &[PgBouncerRuntimeConfigV1],
    ) -> Result<Self, PoolConfigErrorV1> {
        validate_path(path)?;
        let contents = render_entries(entries)?;
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

fn validate_path(path: &Path) -> Result<(), PoolConfigErrorV1> {
    let parent = path.parent().ok_or(PoolConfigErrorV1::FileSystem)?;
    validate_directory(parent)?;
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if trusted_file(&metadata) => Ok(()),
        Ok(_) => Err(PoolConfigErrorV1::FileSystem),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(PoolConfigErrorV1::FileSystem),
    }
}

fn render_entries(entries: &[PgBouncerRuntimeConfigV1]) -> Result<String, PoolConfigErrorV1> {
    let mut entries = entries.to_vec();
    entries.sort_by(|left, right| left.alias().as_str().cmp(right.alias().as_str()));
    if entries
        .windows(2)
        .any(|pair| pair[0].alias().as_str() == pair[1].alias().as_str())
    {
        return Err(PoolConfigErrorV1::Identifier);
    }
    let mut output = String::from("[databases]\n");
    for entry in entries {
        output.push_str(&entry.render_database_entry());
        output.push('\n');
    }
    Ok(output)
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
    // Preserve the inode for Docker Desktop bind mounts. The Storage runtime
    // finishes and syncs this write before it sends PgBouncer a RELOAD command.
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
    let name = path.file_name().and_then(|value| value.to_str());
    name.map(|name| path.with_file_name(format!(".{name}.next")))
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

fn validate_directory(path: &Path) -> Result<(), PoolConfigErrorV1> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| PoolConfigErrorV1::FileSystem)?;
    (metadata.is_dir()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == current_uid()
        && metadata.mode() & 0o777 == PRIVATE_DIRECTORY_MODE)
        .then_some(())
        .ok_or(PoolConfigErrorV1::FileSystem)
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
