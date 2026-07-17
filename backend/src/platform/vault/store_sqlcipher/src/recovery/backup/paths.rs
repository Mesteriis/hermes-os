use std::fs;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::database::store::VaultStoreError;

pub(crate) struct BackupPaths {
    directory: PathBuf,
}

impl BackupPaths {
    pub(crate) fn database(&self) -> PathBuf {
        self.directory.join("vault.db")
    }
    pub(crate) fn anchor(&self) -> PathBuf {
        self.directory.join("vault.anchor")
    }
    pub(crate) fn manifest(&self) -> PathBuf {
        self.directory.join("vault.manifest")
    }
    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }
}

pub(crate) fn create_destination(path: &Path) -> Result<BackupPaths, VaultStoreError> {
    let parent = path.parent().ok_or(VaultStoreError::InsecurePath)?;
    let parent_metadata =
        fs::symlink_metadata(parent).map_err(|_| VaultStoreError::InsecurePath)?;
    if parent_metadata.file_type().is_symlink()
        || !parent_metadata.is_dir()
        || parent_metadata.uid() != unsafe { libc::geteuid() }
        || parent_metadata.mode() & 0o077 != 0
    {
        return Err(VaultStoreError::InsecurePath);
    }
    let name = path.file_name().ok_or(VaultStoreError::InsecurePath)?;
    let directory = fs::canonicalize(parent)
        .map_err(|_| VaultStoreError::InsecurePath)?
        .join(name);
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(&directory)
        .map_err(|_| VaultStoreError::Backup)?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
        .map_err(|_| VaultStoreError::Backup)?;
    Ok(BackupPaths { directory })
}

pub(crate) fn open_destination(path: &Path) -> Result<BackupPaths, VaultStoreError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| VaultStoreError::Backup)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
    {
        return Err(VaultStoreError::Backup);
    }
    let directory = fs::canonicalize(path).map_err(|_| VaultStoreError::Backup)?;
    let paths = BackupPaths { directory };
    for file in [paths.database(), paths.anchor(), paths.manifest()] {
        let metadata = fs::symlink_metadata(file).map_err(|_| VaultStoreError::Backup)?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o077 != 0
        {
            return Err(VaultStoreError::Backup);
        }
    }
    Ok(paths)
}

pub(crate) fn sync_directory(path: &Path) -> Result<(), VaultStoreError> {
    std::fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| VaultStoreError::Backup)
}
