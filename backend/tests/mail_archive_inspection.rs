use std::io::{Cursor, Write};

use hermes_hub_backend::domains::mail::archive_inspection::{
    ArchiveInspectionError, ArchiveInspectionLimits, inspect_zip_bytes,
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (name, bytes) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }

    writer.finish().unwrap().into_inner()
}

#[test]
fn inspects_safe_zip_metadata_without_extracting() {
    let bytes = zip_bytes(&[("docs/readme.txt", b"hello"), ("invoice.pdf", b"pdf bytes")]);

    let report = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap();

    assert_eq!(report.entry_count, 2);
    assert_eq!(report.total_uncompressed_bytes, 14);
    assert_eq!(report.entries[0].normalized_path, "docs/readme.txt");
    assert_eq!(report.entries[0].uncompressed_size, 5);
    assert!(!report.has_nested_archive);
}

#[test]
fn rejects_zip_entries_with_path_traversal() {
    let bytes = zip_bytes(&[("../secret.txt", b"secret")]);

    let err = inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).unwrap_err();

    assert!(matches!(
        err,
        ArchiveInspectionError::UnsafeEntryPath { entry_name }
            if entry_name == "../secret.txt"
    ));
}

#[test]
fn rejects_zip_bombs_by_uncompressed_size_limit() {
    let bytes = zip_bytes(&[("large.txt", b"12345")]);
    let limits = ArchiveInspectionLimits {
        max_uncompressed_bytes: 4,
        ..ArchiveInspectionLimits::default()
    };

    let err = inspect_zip_bytes(&bytes, limits).unwrap_err();

    assert!(matches!(
        err,
        ArchiveInspectionError::UncompressedSizeExceeded { total, limit }
            if total == 5 && limit == 4
    ));
}
