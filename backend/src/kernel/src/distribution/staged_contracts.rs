//! Owns short-lived exact contract files passed to a verified managed process.

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use getrandom::fill;

pub struct StagedRuntimeContracts {
    descriptor_path: PathBuf,
    settings_schema_path: Option<PathBuf>,
    settings_snapshot_path: Option<PathBuf>,
    runtime_configuration_path: Option<PathBuf>,
    host_bridge_configuration_path: Option<PathBuf>,
}

impl StagedRuntimeContracts {
    pub fn stage(
        directory: &Path,
        descriptor_bytes: &[u8],
        settings_schema_bytes: Option<&[u8]>,
    ) -> Result<Self, String> {
        Self::stage_with_runtime_configuration(
            directory,
            descriptor_bytes,
            settings_schema_bytes,
            None,
        )
    }

    pub fn stage_with_runtime_configuration(
        directory: &Path,
        descriptor_bytes: &[u8],
        settings_schema_bytes: Option<&[u8]>,
        runtime_configuration_bytes: Option<&[u8]>,
    ) -> Result<Self, String> {
        Self::stage_with_runtime_and_host_bridge_configuration(
            directory,
            descriptor_bytes,
            settings_schema_bytes,
            runtime_configuration_bytes,
            None,
        )
    }

    pub fn stage_with_runtime_and_host_bridge_configuration(
        directory: &Path,
        descriptor_bytes: &[u8],
        settings_schema_bytes: Option<&[u8]>,
        runtime_configuration_bytes: Option<&[u8]>,
        host_bridge_configuration_bytes: Option<&[u8]>,
    ) -> Result<Self, String> {
        Self::stage_with_runtime_host_bridge_and_settings_snapshot(
            directory,
            descriptor_bytes,
            settings_schema_bytes,
            None,
            runtime_configuration_bytes,
            host_bridge_configuration_bytes,
        )
    }

    pub fn stage_with_runtime_configuration_and_settings_snapshot(
        directory: &Path,
        descriptor_bytes: &[u8],
        settings_schema_bytes: Option<&[u8]>,
        settings_snapshot_bytes: &[u8],
        runtime_configuration_bytes: &[u8],
    ) -> Result<Self, String> {
        Self::stage_with_runtime_host_bridge_and_settings_snapshot(
            directory,
            descriptor_bytes,
            settings_schema_bytes,
            Some(settings_snapshot_bytes),
            Some(runtime_configuration_bytes),
            None,
        )
    }

    pub fn stage_with_runtime_host_bridge_and_settings_snapshot(
        directory: &Path,
        descriptor_bytes: &[u8],
        settings_schema_bytes: Option<&[u8]>,
        settings_snapshot_bytes: Option<&[u8]>,
        runtime_configuration_bytes: Option<&[u8]>,
        host_bridge_configuration_bytes: Option<&[u8]>,
    ) -> Result<Self, String> {
        validate_directory(directory)?;
        if descriptor_bytes.is_empty() {
            return Err("managed runtime descriptor bytes are unavailable".to_owned());
        }
        let suffix = random_suffix()?;
        let descriptor_path = directory.join(format!("descriptor-{suffix}.bin"));
        write_private_file(&descriptor_path, descriptor_bytes)?;
        let settings_schema_path = match settings_schema_bytes {
            Some(bytes) if !bytes.is_empty() => {
                let path = directory.join(format!("settings-{suffix}.bin"));
                if let Err(error) = write_private_file(&path, bytes) {
                    let _ = remove_private_file(&descriptor_path);
                    return Err(error);
                }
                Some(path)
            }
            Some(_) => {
                let _ = remove_private_file(&descriptor_path);
                return Err("managed runtime settings schema bytes are invalid".to_owned());
            }
            None => None,
        };
        let settings_snapshot_path = match settings_snapshot_bytes {
            Some(bytes) if !bytes.is_empty() => {
                let path = directory.join(format!("settings-snapshot-{suffix}.bin"));
                if let Err(error) = write_private_file(&path, bytes) {
                    let _ = remove_optional_files(&descriptor_path, settings_schema_path.as_deref());
                    return Err(error);
                }
                Some(path)
            }
            Some(_) => {
                let _ = remove_optional_files(&descriptor_path, settings_schema_path.as_deref());
                return Err("managed runtime settings snapshot bytes are invalid".to_owned());
            }
            None => None,
        };
        let runtime_configuration_path = match runtime_configuration_bytes {
            Some(bytes) if !bytes.is_empty() => {
                let path = directory.join(format!("configuration-{suffix}.bin"));
                if let Err(error) = write_private_file(&path, bytes) {
                    let _ = remove_staged_files(
                        &descriptor_path,
                        settings_schema_path.as_deref(),
                        settings_snapshot_path.as_deref(),
                        None,
                    );
                    return Err(error);
                }
                Some(path)
            }
            Some(_) => {
                let _ = remove_staged_files(
                    &descriptor_path,
                    settings_schema_path.as_deref(),
                    settings_snapshot_path.as_deref(),
                    None,
                );
                return Err("managed runtime configuration bytes are invalid".to_owned());
            }
            None => None,
        };
        let host_bridge_configuration_path = match host_bridge_configuration_bytes {
            Some(bytes) if !bytes.is_empty() => {
                let path = directory.join(format!("host-bridge-{suffix}.bin"));
                if let Err(error) = write_private_file(&path, bytes) {
                    let _ = remove_staged_files(
                        &descriptor_path,
                        settings_schema_path.as_deref(),
                        settings_snapshot_path.as_deref(),
                        runtime_configuration_path.as_deref(),
                    );
                    return Err(error);
                }
                Some(path)
            }
            Some(_) => {
                let _ = remove_staged_files(
                    &descriptor_path,
                    settings_schema_path.as_deref(),
                    settings_snapshot_path.as_deref(),
                    runtime_configuration_path.as_deref(),
                );
                return Err("managed host bridge configuration bytes are invalid".to_owned());
            }
            None => None,
        };
        Ok(Self {
            descriptor_path,
            settings_schema_path,
            settings_snapshot_path,
            runtime_configuration_path,
            host_bridge_configuration_path,
        })
    }

    #[must_use]
    pub fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
    }

    #[must_use]
    pub fn settings_schema_path(&self) -> Option<&Path> {
        self.settings_schema_path.as_deref()
    }

    #[must_use]
    pub fn settings_snapshot_path(&self) -> Option<&Path> {
        self.settings_snapshot_path.as_deref()
    }

    #[must_use]
    pub fn runtime_configuration_path(&self) -> Option<&Path> {
        self.runtime_configuration_path.as_deref()
    }

    #[must_use]
    pub fn host_bridge_configuration_path(&self) -> Option<&Path> {
        self.host_bridge_configuration_path.as_deref()
    }

    pub fn remove(self) -> Result<(), String> {
        remove_private_file(&self.descriptor_path)?;
        if let Some(path) = self.settings_schema_path {
            remove_private_file(&path)?;
        }
        if let Some(path) = self.settings_snapshot_path {
            remove_private_file(&path)?;
        }
        if let Some(path) = self.runtime_configuration_path {
            remove_private_file(&path)?;
        }
        if let Some(path) = self.host_bridge_configuration_path {
            remove_private_file(&path)?;
        }
        Ok(())
    }
}

fn remove_optional_files(
    descriptor_path: &Path,
    settings_schema_path: Option<&Path>,
) -> Result<(), String> {
    remove_private_file(descriptor_path)?;
    if let Some(path) = settings_schema_path {
        remove_private_file(path)?;
    }
    Ok(())
}

fn remove_staged_files(
    descriptor_path: &Path,
    settings_schema_path: Option<&Path>,
    settings_snapshot_path: Option<&Path>,
    runtime_configuration_path: Option<&Path>,
) -> Result<(), String> {
    remove_optional_files(descriptor_path, settings_schema_path)?;
    if let Some(path) = settings_snapshot_path {
        remove_private_file(path)?;
    }
    if let Some(path) = runtime_configuration_path {
        remove_private_file(path)?;
    }
    Ok(())
}

fn validate_directory(directory: &Path) -> Result<(), String> {
    if !directory.is_absolute() {
        return Err("managed runtime contract directory must be absolute".to_owned());
    }
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let metadata = std::fs::symlink_metadata(directory).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("managed runtime contract directory is invalid".to_owned());
    }
    std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| error.to_string())
}

fn remove_private_file(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("managed runtime contract cleanup requires a regular file".to_owned());
    }
    std::fs::remove_file(path).map_err(|error| error.to_string())
}

fn random_suffix() -> Result<String, String> {
    let mut random = [0_u8; 16];
    fill(&mut random)
        .map_err(|_| "managed runtime contract randomness is unavailable".to_owned())?;
    Ok(random.iter().map(|byte| format!("{byte:02x}")).collect())
}
