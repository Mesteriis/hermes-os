//! Offline ciphertext-only Blob snapshots owned by the Blob runtime.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::storage::root;

const MAGIC: &[u8; 8] = b"HRBLOB01";
const MAX_ENTRIES: usize = 65_536;
const COPY_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobBackupEntryV1 {
    path: String,
    size_bytes: u64,
    sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobBackupManifestV1 {
    entries: Vec<BlobBackupEntryV1>,
}

pub fn export_backup_offline(
    data_dir: &Path,
    destination: &Path,
) -> Result<BlobBackupManifestV1, String> {
    root::validate_private_directory(data_dir).map_err(|_| unavailable())?;
    validate_root_entries(data_dir, false)?;
    let parent = destination.parent().ok_or_else(unavailable)?;
    root::validate_private_directory(parent).map_err(|_| unavailable())?;
    if destination.exists() {
        return Err(unavailable());
    }
    fs::create_dir(destination).map_err(|_| unavailable())?;
    set_private_directory(destination)?;
    let result = export_into(data_dir, destination);
    if result.is_err() {
        let _ = fs::remove_dir_all(destination);
    }
    result
}

pub fn verify_backup_offline(source: &Path) -> Result<BlobBackupManifestV1, String> {
    root::validate_private_directory(source).map_err(|_| unavailable())?;
    validate_root_entries(source, true)?;
    let manifest_path = source.join("manifest.bin");
    let manifest = BlobBackupManifestV1::decode(&read_private_file(&manifest_path)?)?;
    let actual = collect_snapshot_paths(source)?;
    let expected = manifest
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    if actual != expected {
        return Err(unavailable());
    }
    manifest
        .entries
        .iter()
        .try_for_each(|entry| verify_entry(source, entry))?;
    Ok(manifest)
}

pub fn restore_backup_offline(
    source: &Path,
    destination_data_dir: &Path,
) -> Result<BlobBackupManifestV1, String> {
    let manifest = verify_backup_offline(source)?;
    let parent = prepare_absent_target_parent(destination_data_dir)?;
    let staging = create_restore_staging(&parent)?;
    let result = restore_into_staging(source, &staging, &manifest)
        .and_then(|()| publish_staging(&staging, destination_data_dir, &parent));
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result.map(|()| manifest)
}

impl BlobBackupManifestV1 {
    fn encode(entries: Vec<BlobBackupEntryV1>) -> Result<Vec<u8>, String> {
        validate_entries(&entries)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&(entries.len() as u32).to_be_bytes());
        for entry in entries {
            bytes.extend_from_slice(&(entry.path.len() as u16).to_be_bytes());
            bytes.extend_from_slice(entry.path.as_bytes());
            bytes.extend_from_slice(&entry.size_bytes.to_be_bytes());
            bytes.extend_from_slice(&entry.sha256);
        }
        Ok(bytes)
    }

    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = Cursor { bytes, position: 0 };
        if cursor.take(8)? != MAGIC {
            return Err(unavailable());
        }
        let count = u32::from_be_bytes(cursor.array()?) as usize;
        if count > MAX_ENTRIES {
            return Err(unavailable());
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let length = u16::from_be_bytes(cursor.array()?) as usize;
            let path = std::str::from_utf8(cursor.take(length)?)
                .map_err(|_| unavailable())?
                .to_owned();
            entries.push(BlobBackupEntryV1 {
                path,
                size_bytes: u64::from_be_bytes(cursor.array()?),
                sha256: cursor.array()?,
            });
        }
        if cursor.position != bytes.len() {
            return Err(unavailable());
        }
        validate_entries(&entries)?;
        Ok(Self { entries })
    }

    #[must_use]
    pub fn entries(&self) -> &[BlobBackupEntryV1] {
        &self.entries
    }
}

impl BlobBackupEntryV1 {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

fn export_into(data_dir: &Path, destination: &Path) -> Result<BlobBackupManifestV1, String> {
    let mut entries = Vec::new();
    for (directory, extension) in [("content", "blob"), ("metadata", "meta")] {
        let source = data_dir.join(directory);
        root::validate_private_directory(&source).map_err(|_| unavailable())?;
        let target = destination.join(directory);
        fs::create_dir(&target).map_err(|_| unavailable())?;
        set_private_directory(&target)?;
        for entry in fs::read_dir(source).map_err(|_| unavailable())? {
            let path = entry.map_err(|_| unavailable())?.path();
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(unavailable)?;
            if !valid_leaf(name, extension) {
                return Err(unavailable());
            }
            let relative = format!("{directory}/{name}");
            let copied = copy_source_file(&path, &target.join(name), relative)?;
            entries.push(copied);
        }
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = BlobBackupManifestV1 { entries };
    let manifest_bytes = BlobBackupManifestV1::encode(manifest.entries.clone())?;
    write_private_new(&destination.join("manifest.bin"), &manifest_bytes)?;
    sync_directory(destination)?;
    Ok(manifest)
}

fn collect_snapshot_paths(root_path: &Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for (directory, extension) in [("content", "blob"), ("metadata", "meta")] {
        let directory_path = root_path.join(directory);
        root::validate_private_directory(&directory_path).map_err(|_| unavailable())?;
        for entry in fs::read_dir(directory_path).map_err(|_| unavailable())? {
            let path = entry.map_err(|_| unavailable())?.path();
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(unavailable)?;
            if !valid_leaf(name, extension) {
                return Err(unavailable());
            }
            paths.push(format!("{directory}/{name}"));
        }
    }
    paths.sort();
    Ok(paths)
}

fn validate_root_entries(path: &Path, has_manifest: bool) -> Result<(), String> {
    let mut names = fs::read_dir(path)
        .map_err(|_| unavailable())?
        .map(|entry| {
            entry
                .map_err(|_| unavailable())?
                .file_name()
                .into_string()
                .map_err(|_| unavailable())
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let expected = if has_manifest {
        vec!["content", "manifest.bin", "metadata"]
    } else {
        vec!["content", "metadata"]
    };
    (names == expected).then_some(()).ok_or_else(unavailable)
}

fn copy_source_file(
    source: &Path,
    destination: &Path,
    relative: String,
) -> Result<BlobBackupEntryV1, String> {
    let mut input = open_private(source)?;
    let metadata = input.metadata().map_err(|_| unavailable())?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(destination)
        .map_err(|_| unavailable())?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    loop {
        let count = input.read(&mut buffer).map_err(|_| unavailable())?;
        if count == 0 {
            break;
        }
        output
            .write_all(&buffer[..count])
            .map_err(|_| unavailable())?;
        hash.update(&buffer[..count]);
    }
    output.sync_all().map_err(|_| unavailable())?;
    if input.metadata().map_err(|_| unavailable())?.len() != metadata.len() {
        return Err(unavailable());
    }
    Ok(BlobBackupEntryV1 {
        path: relative,
        size_bytes: metadata.len(),
        sha256: hash.finalize().into(),
    })
}

fn verify_entry(root_path: &Path, entry: &BlobBackupEntryV1) -> Result<(), String> {
    let bytes = read_private_file(&root_path.join(&entry.path))?;
    (bytes.len() as u64 == entry.size_bytes && Sha256::digest(&bytes).as_slice() == entry.sha256)
        .then_some(())
        .ok_or_else(unavailable)
}

fn copy_verified_file(
    source: &Path,
    target: &Path,
    entry: &BlobBackupEntryV1,
) -> Result<(), String> {
    verify_entry(source, entry)?;
    let path = target.join(&entry.path);
    copy_source_file(&source.join(&entry.path), &path, entry.path.clone()).and_then(|actual| {
        (actual.size_bytes == entry.size_bytes && actual.sha256 == entry.sha256)
            .then_some(())
            .ok_or_else(unavailable)
    })
}

fn prepare_absent_target_parent(path: &Path) -> Result<PathBuf, String> {
    let parent = path.parent().ok_or_else(unavailable)?;
    root::validate_private_directory(parent).map_err(|_| unavailable())?;
    (!path.exists())
        .then_some(parent.to_path_buf())
        .ok_or_else(unavailable)
}

fn create_restore_staging(parent: &Path) -> Result<PathBuf, String> {
    for attempt in 0..64 {
        let staging = parent.join(format!(".blob-restore-{}-{attempt}", std::process::id()));
        match fs::create_dir(&staging) {
            Ok(()) => {
                set_private_directory(&staging)?;
                for child in ["content", "metadata"] {
                    let directory = staging.join(child);
                    fs::create_dir(&directory).map_err(|_| unavailable())?;
                    set_private_directory(&directory)?;
                }
                return Ok(staging);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err(unavailable()),
        }
    }
    Err(unavailable())
}

fn restore_into_staging(
    source: &Path,
    staging: &Path,
    manifest: &BlobBackupManifestV1,
) -> Result<(), String> {
    for entry in &manifest.entries {
        copy_verified_file(source, staging, entry)?;
    }
    sync_directory(&staging.join("content"))?;
    sync_directory(&staging.join("metadata"))?;
    sync_directory(staging)
}

fn publish_staging(staging: &Path, destination: &Path, parent: &Path) -> Result<(), String> {
    fs::rename(staging, destination).map_err(|_| unavailable())?;
    sync_directory(parent)
}

fn open_private(path: &Path) -> Result<File, String> {
    root::validate_private_regular_file(path).map_err(|_| unavailable())?;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| unavailable())
}

fn read_private_file(path: &Path) -> Result<Vec<u8>, String> {
    let mut file = open_private(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|_| unavailable())?;
    Ok(bytes)
}

fn write_private_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| unavailable())?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| unavailable())
}

fn set_private_directory(path: &Path) -> Result<(), String> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| unavailable())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| unavailable())
}

fn valid_leaf(value: &str, extension: &str) -> bool {
    let stem = value.strip_suffix(&format!(".{extension}")).unwrap_or("");
    stem.len() == 32 && stem.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_entries(entries: &[BlobBackupEntryV1]) -> Result<(), String> {
    if entries.len() > MAX_ENTRIES
        || entries.windows(2).any(|pair| pair[0].path >= pair[1].path)
        || entries
            .iter()
            .any(|entry| !valid_path(&entry.path) || entry.size_bytes == 0)
    {
        return Err(unavailable());
    }
    Ok(())
}

fn valid_path(value: &str) -> bool {
    value.split_once('/').is_some_and(|(directory, name)| {
        matches!(directory, "content" | "metadata")
            && valid_leaf(
                name,
                if directory == "content" {
                    "blob"
                } else {
                    "meta"
                },
            )
    })
}

fn unavailable() -> String {
    "Blob backup is unavailable or invalid".to_owned()
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self.position.checked_add(count).ok_or_else(unavailable)?;
        let result = self.bytes.get(self.position..end).ok_or_else(unavailable)?;
        self.position = end;
        Ok(result)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], String> {
        self.take(N)?.try_into().map_err(|_| unavailable())
    }
}
