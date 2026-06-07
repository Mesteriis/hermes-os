use serde::Serialize;
use thiserror::Error;

use crate::mail_storage::StoredMailAttachmentWithBlob;

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentClassification {
    pub attachment_id: String,
    pub category: AttachmentCategory,
    pub is_executable: bool,
    pub is_archive: bool,
    pub is_document: bool,
    pub risk_level: RiskLevel,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentCategory {
    Invoice,
    Contract,
    LegalDocument,
    TaxDocument,
    IdentityDocument,
    BankDocument,
    Certificate,
    Report,
    Presentation,
    Spreadsheet,
    SourceCode,
    Image,
    Screenshot,
    Archive,
    Unknown,
}

impl AttachmentCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttachmentCategory::Invoice => "invoice",
            AttachmentCategory::Contract => "contract",
            AttachmentCategory::LegalDocument => "legal_document",
            AttachmentCategory::TaxDocument => "tax_document",
            AttachmentCategory::IdentityDocument => "identity_document",
            AttachmentCategory::BankDocument => "bank_document",
            AttachmentCategory::Certificate => "certificate",
            AttachmentCategory::Report => "report",
            AttachmentCategory::Presentation => "presentation",
            AttachmentCategory::Spreadsheet => "spreadsheet",
            AttachmentCategory::SourceCode => "source_code",
            AttachmentCategory::Image => "image",
            AttachmentCategory::Screenshot => "screenshot",
            AttachmentCategory::Archive => "archive",
            AttachmentCategory::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "safe",
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

pub struct AttachmentIntelligenceService;

impl AttachmentIntelligenceService {
    /// Classify an attachment by filename and content type.
    pub fn classify(attachment: &StoredMailAttachmentWithBlob) -> AttachmentClassification {
        let filename = attachment.attachment.filename.as_deref().unwrap_or("");
        let content_type = &attachment.attachment.content_type;
        let filename_lower = filename.to_lowercase();

        let category = classify_by_name_and_type(&filename_lower, content_type);
        let is_executable = is_executable_type(content_type, &filename_lower);
        let is_archive = is_archive_type(content_type, &filename_lower);
        let is_document = is_document_type(content_type, &filename_lower);
        let risk_level = if is_executable {
            RiskLevel::High
        } else if is_archive {
            RiskLevel::Medium
        } else {
            RiskLevel::Safe
        };

        let size_mb = attachment.attachment.size_bytes as f64 / 1_048_576.0;

        AttachmentClassification {
            attachment_id: attachment.attachment.attachment_id.clone(),
            category,
            is_executable,
            is_archive,
            is_document,
            risk_level,
            summary: format!("{} ({:.1} MB) - {}", filename, size_mb, category.as_str()),
        }
    }
}

fn classify_by_name_and_type(filename: &str, content_type: &str) -> AttachmentCategory {
    let lower = filename.to_lowercase();

    if lower.contains("invoice") || lower.contains("factura") || lower.contains("receipt") {
        return AttachmentCategory::Invoice;
    }
    if lower.contains("contract") || lower.contains("agreement") || lower.contains("nda") {
        return AttachmentCategory::Contract;
    }
    if lower.contains("certificate") || lower.contains("cert") {
        return AttachmentCategory::Certificate;
    }
    if lower.contains("tax") || lower.contains("hacienda") || lower.contains("aeat") {
        return AttachmentCategory::TaxDocument;
    }
    if lower.contains("passport") || lower.contains("dni") || lower.contains("nie") {
        return AttachmentCategory::IdentityDocument;
    }
    if lower.contains("report") {
        return AttachmentCategory::Report;
    }
    if lower.contains("presentation") || lower.ends_with(".pptx") || lower.ends_with(".ppt") {
        return AttachmentCategory::Presentation;
    }
    if lower.ends_with(".xlsx") || lower.ends_with(".xls") || lower.ends_with(".csv") {
        return AttachmentCategory::Spreadsheet;
    }
    if lower.ends_with(".rs")
        || lower.ends_with(".py")
        || lower.ends_with(".js")
        || lower.ends_with(".ts")
        || lower.ends_with(".go")
        || lower.ends_with(".java")
        || lower.ends_with(".c")
        || lower.ends_with(".cpp")
    {
        return AttachmentCategory::SourceCode;
    }
    if lower.ends_with(".zip")
        || lower.ends_with(".rar")
        || lower.ends_with(".7z")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tar")
    {
        return AttachmentCategory::Archive;
    }
    if content_type.starts_with("image/") {
        if lower.contains("screenshot") || lower.contains("screen") {
            return AttachmentCategory::Screenshot;
        }
        return AttachmentCategory::Image;
    }
    if content_type == "application/pdf" {
        return AttachmentCategory::Report;
    }

    AttachmentCategory::Unknown
}

fn is_executable_type(content_type: &str, filename: &str) -> bool {
    let executable_types = [
        "application/x-msdownload",
        "application/x-executable",
        "application/x-mach-binary",
        "application/x-sh",
        "application/x-bat",
        "application/x-msi",
    ];
    let executable_exts = [
        ".exe", ".dll", ".sh", ".bat", ".cmd", ".msi", ".app", ".bin",
    ];
    executable_types.contains(&content_type)
        || executable_exts.iter().any(|e| filename.ends_with(e))
}

fn is_archive_type(content_type: &str, filename: &str) -> bool {
    let archive_types = [
        "application/zip",
        "application/x-rar-compressed",
        "application/x-7z-compressed",
        "application/x-tar",
        "application/gzip",
        "application/x-bzip2",
    ];
    let archive_exts = [
        ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz", ".tar.gz",
    ];
    archive_types.contains(&content_type) || archive_exts.iter().any(|e| filename.ends_with(e))
}

fn is_document_type(content_type: &str, filename: &str) -> bool {
    let doc_types = [
        "application/pdf",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "text/plain",
        "text/markdown",
        "text/csv",
    ];
    let doc_exts = [
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".txt", ".md", ".csv",
    ];
    doc_types.contains(&content_type) || doc_exts.iter().any(|e| filename.ends_with(e))
}

#[derive(Debug, Error)]
pub enum AttachmentIntelligenceError {
    #[error("attachment not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail_storage::{
        AttachmentSafetyScanStatus, MailAttachmentDisposition, StoredMailAttachment,
    };
    use chrono::Utc;

    fn test_attachment(
        filename: &str,
        content_type: &str,
        size: i64,
    ) -> StoredMailAttachmentWithBlob {
        StoredMailAttachmentWithBlob {
            attachment: StoredMailAttachment {
                attachment_id: "att:1".into(),
                message_id: "msg:1".into(),
                raw_record_id: "raw:1".into(),
                blob_id: "blob:1".into(),
                provider_attachment_id: "prov:1".into(),
                filename: Some(filename.into()),
                content_type: content_type.into(),
                size_bytes: size,
                sha256: "sha256:abc".into(),
                disposition: MailAttachmentDisposition::Attachment,
                scan_status: AttachmentSafetyScanStatus::NotScanned,
                scan_engine: None,
                scan_checked_at: None,
                scan_summary: None,
                scan_metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            storage_kind: "local_fs".into(),
            storage_path: "/tmp/test".into(),
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
}
