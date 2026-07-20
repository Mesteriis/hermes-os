//! Fail-closed verification of a published recovery-media inventory.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryMediaEntryV1 {
    path: String,
    size_bytes: u64,
    sha256: [u8; 32],
}

pub(crate) struct SignedRecoveryMediaManifestV1 {
    verification_key_id: String,
    raw_manifest_bytes: Vec<u8>,
    signature_raw: [u8; 64],
}

impl SignedRecoveryMediaManifestV1 {
    pub(crate) fn new(
        verification_key_id: String,
        raw_manifest_bytes: Vec<u8>,
        signature_raw: [u8; 64],
    ) -> Result<Self, String> {
        if verification_key_id.is_empty()
            || verification_key_id.len() > 128
            || raw_manifest_bytes.is_empty()
        {
            return Err("signed recovery media manifest is invalid".to_owned());
        }
        Ok(Self {
            verification_key_id,
            raw_manifest_bytes,
            signature_raw,
        })
    }

    pub(crate) fn verify(
        &self,
        expected_key_id: &str,
        public_key_sec1: &[u8],
    ) -> Result<(), String> {
        if self.verification_key_id != expected_key_id {
            return Err("recovery media verification key is not pinned".to_owned());
        }
        let key = VerifyingKey::from_sec1_bytes(public_key_sec1)
            .map_err(|_| "recovery media verification key is invalid".to_owned())?;
        let signature = Signature::from_slice(&self.signature_raw)
            .map_err(|_| "recovery media signature is invalid".to_owned())?;
        key.verify(&self.raw_manifest_bytes, &signature)
            .map_err(|_| "recovery media signature verification failed".to_owned())
    }
}

impl RecoveryMediaEntryV1 {
    pub(crate) fn new(path: String, size_bytes: u64, sha256: [u8; 32]) -> Result<Self, String> {
        if !valid_relative_path(&path) {
            return Err("recovery media path is invalid".to_owned());
        }
        Ok(Self {
            path,
            size_bytes,
            sha256,
        })
    }
}

/// Ensures that recovery media has exactly the signed manifest inventory before
/// any restore target is created. The caller owns signature verification.
pub(crate) fn verify_inventory(
    root: &Path,
    expected: &[RecoveryMediaEntryV1],
) -> Result<(), String> {
    let expected_paths = expected
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    if expected_paths.len() != expected.len() {
        return Err("recovery media inventory contains duplicate paths".to_owned());
    }
    let actual_paths = collect_regular_files(root)?;
    if actual_paths != expected_paths.into_iter().map(str::to_owned).collect() {
        return Err("recovery media inventory does not match manifest".to_owned());
    }
    expected
        .iter()
        .try_for_each(|entry| verify_entry(root, entry))
}

fn verify_entry(root: &Path, entry: &RecoveryMediaEntryV1) -> Result<(), String> {
    let path = root.join(&entry.path);
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|_| "recovery media file is missing".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() != entry.size_bytes
    {
        return Err("recovery media file is invalid".to_owned());
    }
    let mut file =
        File::open(path).map_err(|_| "recovery media file cannot be opened".to_owned())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| "recovery media file cannot be read".to_owned())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if <[u8; 32]>::from(digest.finalize()) != entry.sha256 {
        return Err("recovery media file digest does not match manifest".to_owned());
    }
    Ok(())
}

fn collect_regular_files(root: &Path) -> Result<BTreeSet<String>, String> {
    let metadata = std::fs::symlink_metadata(root)
        .map_err(|_| "recovery media root is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("recovery media root is invalid".to_owned());
    }
    let mut files = BTreeSet::new();
    collect(root, root, &mut files)?;
    Ok(files)
}

fn collect(root: &Path, directory: &Path, files: &mut BTreeSet<String>) -> Result<(), String> {
    for entry in std::fs::read_dir(directory)
        .map_err(|_| "recovery media directory cannot be read".to_owned())?
    {
        let path = entry
            .map_err(|_| "recovery media entry cannot be read".to_owned())?
            .path();
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| "recovery media entry is unavailable".to_owned())?;
        if metadata.file_type().is_symlink() {
            return Err("recovery media must not contain symlinks".to_owned());
        }
        if metadata.is_dir() {
            collect(root, &path, files)?;
            continue;
        }
        if !metadata.is_file() {
            return Err("recovery media must contain regular files only".to_owned());
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "recovery media path is invalid".to_owned())?;
        files.insert(relative.to_string_lossy().into_owned());
    }
    Ok(())
}

fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && PathBuf::from(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}
