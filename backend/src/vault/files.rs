use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::errors::HostVaultError;

pub(super) fn write_secure_file(path: &Path, bytes: &[u8]) -> Result<(), HostVaultError> {
    if let Some(parent) = path.parent() {
        ensure_secure_dir(parent)?;
    }
    let temp_path = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    ensure_secure_file(&temp_path)?;
    fs::rename(&temp_path, path)?;
    ensure_secure_file(path)?;
    Ok(())
}

pub(super) fn ensure_secure_dir(path: &Path) -> Result<(), HostVaultError> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub(super) fn ensure_secure_file(path: &Path) -> Result<(), HostVaultError> {
    if !path.exists() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub(super) fn guard_release_dev_mode(dev_mode: bool) -> Result<(), HostVaultError> {
    if dev_mode && !cfg!(debug_assertions) {
        return Err(HostVaultError::DevModeForbiddenInRelease);
    }
    Ok(())
}
