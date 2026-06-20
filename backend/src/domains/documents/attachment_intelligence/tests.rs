use super::file_kinds::{is_archive_type, is_executable_type};
use super::*;

fn test_attachment(filename: &str, content_type: &str, size: i64) -> AttachmentIntelligenceInput {
    AttachmentIntelligenceInput {
        attachment_id: "att:1".into(),
        filename: Some(filename.into()),
        content_type: content_type.into(),
        size_bytes: size,
    }
}

#[test]
fn classify_invoice_by_filename() {
    let att = test_attachment("Invoice_2026_001.pdf", "application/pdf", 100_000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.category.as_str(), "invoice");
}

#[test]
fn classify_contract_by_filename() {
    let att = test_attachment(
        "NDA_Acme_Corp.docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        50_000,
    );
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.category.as_str(), "contract");
}

#[test]
fn classify_archive_by_extension() {
    let att = test_attachment("documents.zip", "application/zip", 1_000_000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.category.as_str(), "archive");
    assert_eq!(result.risk_level.as_str(), "medium");
}

#[test]
fn classify_executable_as_high_risk() {
    let att = test_attachment("setup.exe", "application/x-msdownload", 5_000_000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert!(result.is_executable);
    assert_eq!(result.risk_level.as_str(), "high");
}

#[test]
fn classify_image_as_safe() {
    let att = test_attachment("photo.jpg", "image/jpeg", 200_000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.risk_level.as_str(), "safe");
}

#[test]
fn classify_source_code() {
    let att = test_attachment("main.rs", "text/plain", 5000);
    let result = AttachmentIntelligenceService::classify(&att);
    assert_eq!(result.category.as_str(), "source_code");
}

#[test]
fn is_executable_detects_exe() {
    assert!(is_executable_type("application/x-msdownload", "setup.exe"));
    assert!(!is_executable_type("application/pdf", "doc.pdf"));
}

#[test]
fn is_archive_detects_zip() {
    assert!(is_archive_type("application/zip", "archive.zip"));
    assert!(!is_archive_type("application/pdf", "doc.pdf"));
}
