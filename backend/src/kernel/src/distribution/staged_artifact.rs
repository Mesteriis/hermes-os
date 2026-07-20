//! Creates a private immutable execution copy after exact-byte verification.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct StagedNativeArtifact {
    path: PathBuf,
}

impl StagedNativeArtifact {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn remove(self) -> Result<(), String> {
        let metadata = std::fs::symlink_metadata(&self.path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("staged artifact cleanup requires a regular non-symlink file".to_owned());
        }
        std::fs::remove_file(self.path).map_err(|error| error.to_string())
    }
}

pub fn stage(
    source: &Path,
    launch_directory: &Path,
    artifact_name: &str,
    expected_sha256: &[u8; 32],
) -> Result<StagedNativeArtifact, String> {
    let source_metadata = validate_stage_paths(source, launch_directory, artifact_name)?;
    let destination = launch_directory.join(artifact_name);
    let result = copy_verified(source, &source_metadata, &destination, expected_sha256);
    if result.is_err() {
        let _ = std::fs::remove_file(&destination);
    }
    result
}

fn validate_stage_paths(
    source: &Path,
    launch_directory: &Path,
    artifact_name: &str,
) -> Result<std::fs::Metadata, String> {
    if !source.is_absolute() || !launch_directory.is_absolute() {
        return Err("staged artifact paths must be absolute".to_owned());
    }
    if artifact_name.is_empty()
        || matches!(artifact_name, "." | "..")
        || artifact_name.contains(['/', '\\'])
    {
        return Err("staged artifact name is invalid".to_owned());
    }
    let source_metadata = std::fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_file() {
        return Err("staged artifact source must be a regular non-symlink file".to_owned());
    }
    std::fs::create_dir_all(launch_directory).map_err(|error| error.to_string())?;
    let directory_metadata =
        std::fs::symlink_metadata(launch_directory).map_err(|error| error.to_string())?;
    if directory_metadata.file_type().is_symlink() || !directory_metadata.is_dir() {
        return Err("staged artifact directory must be a non-symlink directory".to_owned());
    }
    std::fs::set_permissions(launch_directory, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    Ok(source_metadata)
}

fn copy_verified(
    source: &Path,
    source_metadata: &std::fs::Metadata,
    destination: &Path,
    expected_sha256: &[u8; 32],
) -> Result<StagedNativeArtifact, String> {
    let mut input = File::open(source).map_err(|error| error.to_string())?;
    let opened_metadata = input.metadata().map_err(|error| error.to_string())?;
    if !same_file(source_metadata, &opened_metadata) {
        return Err("staged artifact source changed while it was opened".to_owned());
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o700)
        .open(destination)
        .map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = input.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        output
            .write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
        digest.update(&buffer[..read]);
    }
    let opened_after = input.metadata().map_err(|error| error.to_string())?;
    let source_after = std::fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if !same_file(&opened_metadata, &opened_after) || !same_file(&opened_metadata, &source_after) {
        return Err("staged artifact source changed while it was read".to_owned());
    }
    output.sync_all().map_err(|error| error.to_string())?;
    if digest.finalize().as_slice() != expected_sha256 {
        return Err("staged artifact digest does not match manifest".to_owned());
    }
    drop(output);
    std::fs::set_permissions(destination, std::fs::Permissions::from_mode(0o500))
        .map_err(|error| error.to_string())?;
    Ok(StagedNativeArtifact {
        path: destination.to_owned(),
    })
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
