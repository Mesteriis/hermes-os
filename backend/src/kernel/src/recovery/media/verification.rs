//! fd-bound verification of a published recovery-media inventory.

use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::encryption::{DecryptedRecoveryPayload, RecoveryMediaEncryptionKey};
use super::format::{RecoveryMediaEntryV1, RecoveryMediaManifestV1};
use super::layout::{MANIFEST_FILE, PAYLOAD_DIRECTORY, SIGNATURE_FILE};
use super::signature::SignedRecoveryMediaManifestV1;

const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_SIGNATURE_BYTES: u64 = 512;

/// Ensures that recovery media has exactly the signed manifest inventory before
/// any restore target is created. The caller owns signature verification.
pub(crate) fn verify_inventory(
    root: &Path,
    expected: &[RecoveryMediaEntryV1],
) -> Result<(), String> {
    let expected_paths = expected
        .iter()
        .map(RecoveryMediaEntryV1::path)
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
) -> Result<RecoveryMediaManifestV1, String> {
    let manifest = signed.verify_and_decode(expected_key_id, public_key_sec1)?;
    verify_inventory(root, manifest.entries())?;
    Ok(manifest)
}

pub(crate) fn verify_published_recovery_media(
    root: &Path,
    expected_key_id: &str,
    public_key_sec1: &[u8],
) -> Result<RecoveryMediaManifestV1, String> {
    validate_published_root(root)?;
    let manifest = read_private_file(&root.join(MANIFEST_FILE), MAX_MANIFEST_BYTES)?;
    let signature = read_private_file(&root.join(SIGNATURE_FILE), MAX_SIGNATURE_BYTES)?;
    let signed = SignedRecoveryMediaManifestV1::decode(manifest, &signature)?;
    verify_signed_inventory(
        &root.join(PAYLOAD_DIRECTORY),
        &signed,
        expected_key_id,
        public_key_sec1,
    )
}

pub(crate) fn open_verified_recovery_media(
    root: &Path,
    expected_key_id: &str,
    public_key_sec1: &[u8],
    workspace_parent: &Path,
    encryption_key: &RecoveryMediaEncryptionKey,
) -> Result<(RecoveryMediaManifestV1, DecryptedRecoveryPayload), String> {
    let manifest = verify_published_recovery_media(root, expected_key_id, public_key_sec1)?;
    let payload = DecryptedRecoveryPayload::create(
        &root.join(PAYLOAD_DIRECTORY),
        manifest.entries(),
        workspace_parent,
        encryption_key,
    )?;
    Ok((manifest, payload))
}

fn validate_published_root(root: &Path) -> Result<(), String> {
    ensure_private_directory(root, "recovery media root")?;
    ensure_private_directory(&root.join(PAYLOAD_DIRECTORY), "recovery media payload")?;
    let actual = std::fs::read_dir(root)
        .map_err(|_| "recovery media root cannot be read".to_owned())?
        .map(|entry| {
            entry
                .map_err(|_| "recovery media root cannot be read".to_owned())?
                .file_name()
                .into_string()
                .map_err(|_| "recovery media root entry is invalid".to_owned())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let expected = [PAYLOAD_DIRECTORY, MANIFEST_FILE, SIGNATURE_FILE]
        .into_iter()
        .map(str::to_owned)
        .collect();
    (actual == expected)
        .then_some(())
        .ok_or_else(|| "recovery media root layout is invalid".to_owned())
}

fn ensure_private_directory(path: &Path, label: &str) -> Result<(), String> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| format!("{label} is unavailable"))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != current_uid()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(format!("{label} is invalid"));
    }
    Ok(())
}

fn read_private_file(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, String> {
    let before = std::fs::symlink_metadata(path)
        .map_err(|_| "recovery media metadata is unavailable".to_owned())?;
    if before.file_type().is_symlink()
        || !before.is_file()
        || before.uid() != current_uid()
        || before.permissions().mode() & 0o077 != 0
        || before.len() == 0
        || before.len() > maximum_bytes
    {
        return Err("recovery media metadata is invalid".to_owned());
    }
    let mut file = open_no_follow(path)?;
    let opened = file
        .metadata()
        .map_err(|_| "recovery media metadata cannot be inspected".to_owned())?;
    if !same_file(&before, &opened) {
        return Err("recovery media metadata changed while opening".to_owned());
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(opened.len())
            .map_err(|_| "recovery media metadata is too large".to_owned())?,
    );
    file.read_to_end(&mut bytes)
        .map_err(|_| "recovery media metadata cannot be read".to_owned())?;
    let after = file
        .metadata()
        .map_err(|_| "recovery media metadata cannot be inspected".to_owned())?;
    let path_after = std::fs::symlink_metadata(path)
        .map_err(|_| "recovery media metadata is unavailable".to_owned())?;
    if !same_file(&opened, &after) || !same_file(&opened, &path_after) {
        return Err("recovery media metadata changed while reading".to_owned());
    }
    Ok(bytes)
}

fn verify_entry(root: &Path, entry: &RecoveryMediaEntryV1) -> Result<(), String> {
    let path = root.join(entry.path());
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|_| "recovery media file is missing".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() != entry.size_bytes()
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
    if <[u8; 32]>::from(digest.finalize()) != *entry.sha256() {
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

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}
