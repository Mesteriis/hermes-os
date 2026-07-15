use std::io::Cursor;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zip::{ZipArchive, result::ZipError};

const DEFAULT_MAX_ARCHIVE_BYTES: u64 = 100 * 1024 * 1024;
const DEFAULT_MAX_UNCOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;
const DEFAULT_MAX_ENTRIES: usize = 1_000;
const DEFAULT_MAX_DEPTH: usize = 3;
const ARCHIVE_INSPECTION_CACHE_KEY: &str = "archive_inspection";
const ARCHIVE_INSPECTION_CACHE_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionLimits {
    pub max_archive_bytes: u64,
    pub max_uncompressed_bytes: u64,
    pub max_entries: usize,
    pub max_depth: usize,
}

impl Default for ArchiveInspectionLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: DEFAULT_MAX_ARCHIVE_BYTES,
            max_uncompressed_bytes: DEFAULT_MAX_UNCOMPRESSED_BYTES,
            max_entries: DEFAULT_MAX_ENTRIES,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ArchiveInspectionReport {
    pub archive_kind: String,
    pub entry_count: usize,
    pub total_uncompressed_bytes: u64,
    pub has_nested_archive: bool,
    pub entries: Vec<ArchiveEntryInspection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ArchiveEntryInspection {
    pub name: String,
    pub normalized_path: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub is_dir: bool,
    pub is_nested_archive: bool,
}

/// Returns a cached report only when it was produced for the current blob.
pub fn cached_archive_inspection_report(
    scan_metadata: &serde_json::Value,
    source_sha256: &str,
) -> Option<ArchiveInspectionReport> {
    let cached = scan_metadata.get(ARCHIVE_INSPECTION_CACHE_KEY)?;
    if cached.get("version")?.as_u64()? != u64::from(ARCHIVE_INSPECTION_CACHE_VERSION)
        || cached.get("source_sha256")?.as_str()? != source_sha256
    {
        return None;
    }

    serde_json::from_value(cached.get("report")?.clone()).ok()
}

/// Produces the metadata fragment persisted alongside the attachment scan state.
pub fn archive_inspection_cache_metadata(
    source_sha256: &str,
    report: &ArchiveInspectionReport,
) -> serde_json::Value {
    serde_json::json!({
        "version": ARCHIVE_INSPECTION_CACHE_VERSION,
        "source_sha256": source_sha256,
        "report": report,
    })
}

pub fn inspect_zip_bytes(
    bytes: &[u8],
    limits: ArchiveInspectionLimits,
) -> Result<ArchiveInspectionReport, ArchiveInspectionError> {
    if bytes.len() as u64 > limits.max_archive_bytes {
        return Err(ArchiveInspectionError::ArchiveSizeExceeded {
            size: bytes.len() as u64,
            limit: limits.max_archive_bytes,
        });
    }

    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;
    let entry_count = archive.len();
    if entry_count > limits.max_entries {
        return Err(ArchiveInspectionError::EntryCountExceeded {
            count: entry_count,
            limit: limits.max_entries,
        });
    }

    let mut entries = Vec::with_capacity(entry_count);
    let mut total_uncompressed_bytes = 0_u64;
    let mut has_nested_archive = false;

    for index in 0..entry_count {
        let file = archive.by_index(index).map_err(encrypted_archive_error)?;
        let name = file.name().to_owned();
        if file.encrypted() {
            return Err(ArchiveInspectionError::EncryptedEntry { entry_name: name });
        }
        let normalized_path = normalize_archive_entry_path(&name)?;
        let depth = path_depth(&normalized_path);
        if depth > limits.max_depth {
            return Err(ArchiveInspectionError::EntryDepthExceeded {
                entry_name: name,
                depth,
                limit: limits.max_depth,
            });
        }

        let uncompressed_size = file.size();
        total_uncompressed_bytes = total_uncompressed_bytes
            .checked_add(uncompressed_size)
            .ok_or(ArchiveInspectionError::UncompressedSizeExceeded {
                total: u64::MAX,
                limit: limits.max_uncompressed_bytes,
            })?;
        if total_uncompressed_bytes > limits.max_uncompressed_bytes {
            return Err(ArchiveInspectionError::UncompressedSizeExceeded {
                total: total_uncompressed_bytes,
                limit: limits.max_uncompressed_bytes,
            });
        }

        let is_nested_archive = is_archive_path(&normalized_path);
        if is_nested_archive {
            return Err(ArchiveInspectionError::NestedArchiveNotAllowed { entry_name: name });
        }
        has_nested_archive |= is_nested_archive;
        entries.push(ArchiveEntryInspection {
            name,
            normalized_path,
            compressed_size: file.compressed_size(),
            uncompressed_size,
            is_dir: file.is_dir(),
            is_nested_archive,
        });
    }

    Ok(ArchiveInspectionReport {
        archive_kind: "zip".to_owned(),
        entry_count,
        total_uncompressed_bytes,
        has_nested_archive,
        entries,
    })
}

fn normalize_archive_entry_path(name: &str) -> Result<String, ArchiveInspectionError> {
    let normalized = name.trim().replace('\\', "/");
    if normalized.is_empty() || normalized.starts_with('/') {
        return Err(ArchiveInspectionError::UnsafeEntryPath {
            entry_name: name.to_owned(),
        });
    }

    let mut parts = Vec::new();
    for part in normalized.split('/') {
        let part = part.trim();
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains(':') {
            return Err(ArchiveInspectionError::UnsafeEntryPath {
                entry_name: name.to_owned(),
            });
        }
        parts.push(part);
    }

    if parts.is_empty() {
        return Err(ArchiveInspectionError::UnsafeEntryPath {
            entry_name: name.to_owned(),
        });
    }

    Ok(parts.join("/"))
}

fn path_depth(path: &str) -> usize {
    path.split('/').filter(|part| !part.is_empty()).count()
}

fn is_archive_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".zip") || lower.ends_with(".rar") || lower.ends_with(".7z")
}

fn encrypted_archive_error(error: ZipError) -> ArchiveInspectionError {
    match error {
        ZipError::UnsupportedArchive(message)
            if message.to_ascii_lowercase().contains("password required") =>
        {
            ArchiveInspectionError::EncryptedArchive
        }
        error => ArchiveInspectionError::Zip(error),
    }
}

#[derive(Debug, Error)]
pub enum ArchiveInspectionError {
    #[error("archive size {size} exceeds limit {limit}")]
    ArchiveSizeExceeded { size: u64, limit: u64 },
    #[error("archive entry count {count} exceeds limit {limit}")]
    EntryCountExceeded { count: usize, limit: usize },
    #[error("archive uncompressed size {total} exceeds limit {limit}")]
    UncompressedSizeExceeded { total: u64, limit: u64 },
    #[error("archive entry {entry_name} depth {depth} exceeds limit {limit}")]
    EntryDepthExceeded {
        entry_name: String,
        depth: usize,
        limit: usize,
    },
    #[error("unsafe archive entry path: {entry_name}")]
    UnsafeEntryPath { entry_name: String },
    #[error("password-protected archive entry is not allowed: {entry_name}")]
    EncryptedEntry { entry_name: String },
    #[error("password-protected archive is not allowed")]
    EncryptedArchive,
    #[error(
        "nested archive entry is not allowed without recursive sandbox inspection: {entry_name}"
    )]
    NestedArchiveNotAllowed { entry_name: String },
    #[error("zip inspection failed: {0}")]
    Zip(#[from] zip::result::ZipError),
}
