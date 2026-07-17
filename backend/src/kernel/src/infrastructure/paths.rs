//! Canonical private data/runtime path preparation.

use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::infrastructure::filesystem::{
    acquire_runtime_directory_lock, ensure_not_symlink, ensure_owner_private_directory,
    resolve_runtime_directory,
};

pub(crate) fn prepare_runtime_directories(data_dir: &Path) -> Result<PathBuf, String> {
    let data_dir_existed = data_dir.exists();
    prepare_directory(data_dir, data_dir_existed, "data directory")?;
    let canonical_data_dir = data_dir.canonicalize().map_err(|error| error.to_string())?;
    let runtime_dir = resolve_runtime_directory(&canonical_data_dir)?;
    let runtime_dir_existed = runtime_dir.exists();
    prepare_directory(&runtime_dir, runtime_dir_existed, "runtime directory")?;
    Ok(canonical_data_dir)
}

pub(crate) fn prepare_offline_control_store(
    data_dir_override: Option<PathBuf>,
) -> Result<(PathBuf, PathBuf, File), String> {
    let data_dir = data_dir_override
        .filter(|path| path.is_absolute())
        .ok_or_else(|| {
            "offline control-store operations require an explicit absolute --data-dir".to_owned()
        })?;
    ensure_not_symlink(&data_dir, "data directory")?;
    if !data_dir.exists() {
        return Err("offline control-store data directory does not exist".to_owned());
    }
    ensure_owner_private_directory(&data_dir)?;
    let data_dir = data_dir.canonicalize().map_err(|error| error.to_string())?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let runtime_dir_existed = runtime_dir.exists();
    prepare_directory(&runtime_dir, runtime_dir_existed, "runtime directory")?;
    let lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store_path = data_dir.join("kernel-control-store.sqlite");
    Ok((data_dir, store_path, lock))
}

fn prepare_directory(path: &Path, existed: bool, label: &str) -> Result<(), String> {
    ensure_not_symlink(path, label)?;
    std::fs::create_dir_all(path).map_err(|error| error.to_string())?;
    ensure_not_symlink(path, label)?;
    if existed {
        ensure_owner_private_directory(path)
    } else {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())
    }
}
