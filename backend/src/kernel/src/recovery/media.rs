//! Fail-closed verification of a published recovery-media inventory.

use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};

use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};

const MANIFEST_MAGIC: &[u8; 8] = b"HRMEDIA1";
const MAX_ENTRIES: usize = 256;
const MAX_PATH_BYTES: usize = 512;

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

pub(crate) struct RecoveryMediaManifestV1 {
    entries: Vec<RecoveryMediaEntryV1>,
}

impl RecoveryMediaManifestV1 {
    pub(crate) fn encode(entries: Vec<RecoveryMediaEntryV1>) -> Result<Vec<u8>, String> {
        validate_entries(&entries)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MANIFEST_MAGIC);
        bytes.extend_from_slice(&(entries.len() as u16).to_be_bytes());
        for entry in entries {
            let path = entry.path.as_bytes();
            bytes.extend_from_slice(&(path.len() as u16).to_be_bytes());
            bytes.extend_from_slice(path);
            bytes.extend_from_slice(&entry.size_bytes.to_be_bytes());
            bytes.extend_from_slice(&entry.sha256);
        }
        Ok(bytes)
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = Cursor::new(bytes);
        if cursor.take(8)? != MANIFEST_MAGIC {
            return Err("recovery media manifest is invalid".to_owned());
        }
        let count = usize::from(u16::from_be_bytes(cursor.array()?));
        if count == 0 || count > MAX_ENTRIES {
            return Err("recovery media manifest is invalid".to_owned());
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(decode_entry(&mut cursor)?);
        }
        if !cursor.remaining().is_empty() {
            return Err("recovery media manifest is invalid".to_owned());
        }
        validate_entries(&entries)?;
        Ok(Self { entries })
    }

    fn entries(&self) -> &[RecoveryMediaEntryV1] {
        &self.entries
    }
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

    pub(crate) fn verify_and_decode(
        &self,
        expected_key_id: &str,
        public_key_sec1: &[u8],
    ) -> Result<RecoveryMediaManifestV1, String> {
        self.verify(expected_key_id, public_key_sec1)?;
        RecoveryMediaManifestV1::decode(&self.raw_manifest_bytes)
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

pub(crate) fn verify_signed_inventory(
    root: &Path,
    signed: &SignedRecoveryMediaManifestV1,
    expected_key_id: &str,
    public_key_sec1: &[u8],
) -> Result<(), String> {
    let manifest = signed.verify_and_decode(expected_key_id, public_key_sec1)?;
    verify_inventory(root, manifest.entries())
}

fn decode_entry(cursor: &mut Cursor<'_>) -> Result<RecoveryMediaEntryV1, String> {
    let length = usize::from(u16::from_be_bytes(cursor.array()?));
    if length == 0 || length > MAX_PATH_BYTES {
        return Err("recovery media manifest is invalid".to_owned());
    }
    let path = std::str::from_utf8(cursor.take(length)?)
        .map_err(|_| "recovery media manifest is invalid".to_owned())?
        .to_owned();
    let size_bytes = u64::from_be_bytes(cursor.array()?);
    let sha256 = cursor.array()?;
    RecoveryMediaEntryV1::new(path, size_bytes, sha256)
        .map_err(|_| "recovery media manifest is invalid".to_owned())
}

fn validate_entries(entries: &[RecoveryMediaEntryV1]) -> Result<(), String> {
    if entries.is_empty() || entries.len() > MAX_ENTRIES {
        return Err("recovery media manifest is invalid".to_owned());
    }
    let paths = entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>();
    if paths.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("recovery media manifest entries are not canonical".to_owned());
    }
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    fn take(&mut self, length: usize) -> Result<&'a [u8], String> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| "recovery media manifest is invalid".to_owned())?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| "recovery media manifest is invalid".to_owned())?;
        self.position = end;
        Ok(bytes)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], String> {
        self.take(N)?
            .try_into()
            .map_err(|_| "recovery media manifest is invalid".to_owned())
    }
    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.position..]
    }
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
    let mut file = open_no_follow(&path)?;
    let opened = file
        .metadata()
        .map_err(|_| "recovery media file cannot be inspected".to_owned())?;
    if !same_file(&metadata, &opened) {
        return Err("recovery media file changed while it was opened".to_owned());
    }
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
    let after = file
        .metadata()
        .map_err(|_| "recovery media file cannot be inspected".to_owned())?;
    let path_after = std::fs::symlink_metadata(&path)
        .map_err(|_| "recovery media file is unavailable".to_owned())?;
    if !same_file(&opened, &after) || !same_file(&opened, &path_after) {
        return Err("recovery media file changed while it was read".to_owned());
    }
    if <[u8; 32]>::from(digest.finalize()) != entry.sha256 {
        return Err("recovery media file digest does not match manifest".to_owned());
    }
    Ok(())
}

fn open_no_follow(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| "recovery media file cannot be opened".to_owned())
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
