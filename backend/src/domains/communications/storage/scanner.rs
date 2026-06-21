use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::errors::{AttachmentSafetyScanError, MailStorageError};
use super::validation::validate_non_empty;

#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentSafetyScanReport {
    pub status: AttachmentSafetyScanStatus,
    pub engine: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub summary: Option<String>,
    pub metadata: Value,
}

impl AttachmentSafetyScanReport {
    pub fn not_scanned() -> Self {
        Self {
            status: AttachmentSafetyScanStatus::NotScanned,
            engine: None,
            checked_at: None,
            summary: None,
            metadata: json!({}),
        }
    }

    pub(crate) fn validate(&self) -> Result<Self, MailStorageError> {
        let engine = self
            .engine
            .as_deref()
            .map(|value| validate_non_empty("scan_engine", value))
            .transpose()?;
        let summary = self
            .summary
            .as_deref()
            .map(|value| validate_non_empty("scan_summary", value))
            .transpose()?;
        if !self.metadata.is_object() {
            return Err(MailStorageError::NonObjectJson("scan_metadata"));
        }

        if self.status == AttachmentSafetyScanStatus::NotScanned
            && (engine.is_some() || self.checked_at.is_some() || summary.is_some())
        {
            return Err(MailStorageError::InvalidNotScannedReport);
        }

        Ok(Self {
            status: self.status,
            engine,
            checked_at: self.checked_at,
            summary,
            metadata: self.metadata.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSafetyScanStatus {
    NotScanned,
    Clean,
    Suspicious,
    Malicious,
    Failed,
}

impl AttachmentSafetyScanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotScanned => "not_scanned",
            Self::Clean => "clean",
            Self::Suspicious => "suspicious",
            Self::Malicious => "malicious",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for AttachmentSafetyScanStatus {
    type Error = MailStorageError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not_scanned" => Ok(Self::NotScanned),
            "clean" => Ok(Self::Clean),
            "suspicious" => Ok(Self::Suspicious),
            "malicious" => Ok(Self::Malicious),
            "failed" => Ok(Self::Failed),
            other => Err(MailStorageError::InvalidScanStatus(other.to_owned())),
        }
    }
}

pub struct AttachmentSafetyScanRequest<'a> {
    pub provider_attachment_id: &'a str,
    pub filename: Option<&'a str>,
    pub content_type: &'a str,
    pub size_bytes: i64,
    pub sha256: &'a str,
    pub storage_kind: &'a str,
    pub storage_path: &'a str,
    pub bytes: &'a [u8],
}

pub trait AttachmentSafetyScanner {
    fn scan(
        &self,
        request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicAttachmentSafetyScanner;

impl AttachmentSafetyScanner for HeuristicAttachmentSafetyScanner {
    fn scan(
        &self,
        request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
        let extension = normalized_extension(request.filename);
        let content_type = normalized_content_type(request.content_type);
        let mut reasons = Vec::new();
        let mut status = AttachmentSafetyScanStatus::NotScanned;

        if has_executable_magic(request.bytes) {
            status = AttachmentSafetyScanStatus::Malicious;
            reasons.push("executable_magic");
        }

        if let Some(extension) = extension.as_deref() {
            if is_active_content_extension(extension) {
                status = AttachmentSafetyScanStatus::Malicious;
                reasons.push("active_content_extension");
            } else if is_macro_document_extension(extension) {
                status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
                reasons.push("macro_enabled_document_extension");
            }
        }

        if let Some(extension) = extension.as_deref()
            && is_mime_extension_mismatch(&content_type, extension)
        {
            status = max_scan_status(status, AttachmentSafetyScanStatus::Suspicious);
            reasons.push("mime_extension_mismatch");
        }

        if status == AttachmentSafetyScanStatus::NotScanned {
            return Ok(AttachmentSafetyScanReport::not_scanned());
        }

        Ok(AttachmentSafetyScanReport {
            status,
            engine: Some("hermes_heuristic_v1".to_owned()),
            checked_at: Some(Utc::now()),
            summary: Some(scan_summary(status).to_owned()),
            metadata: json!({
                "reasons": reasons,
                "content_type": content_type,
                "filename_extension": extension,
                "size_bytes": request.size_bytes,
            }),
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoopAttachmentSafetyScanner;

impl AttachmentSafetyScanner for NoopAttachmentSafetyScanner {
    fn scan(
        &self,
        _request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
        Ok(AttachmentSafetyScanReport::not_scanned())
    }
}

fn normalized_extension(filename: Option<&str>) -> Option<String> {
    let filename = filename?.trim();
    let basename = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .trim();
    let (_, extension) = basename.rsplit_once('.')?;
    let extension = extension.trim().to_ascii_lowercase();
    (!extension.is_empty()).then_some(extension)
}

fn normalized_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase()
}

fn has_executable_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(b"MZ") || bytes.starts_with(b"\x7fELF")
}

fn is_active_content_extension(extension: &str) -> bool {
    matches!(
        extension,
        "app"
            | "bat"
            | "cmd"
            | "com"
            | "dll"
            | "dmg"
            | "exe"
            | "hta"
            | "jar"
            | "jse"
            | "js"
            | "msi"
            | "ps1"
            | "scr"
            | "vbe"
            | "vbs"
            | "wsf"
    )
}

fn is_macro_document_extension(extension: &str) -> bool {
    matches!(
        extension,
        "docm" | "dotm" | "xlsm" | "xltm" | "pptm" | "potm"
    )
}

fn is_mime_extension_mismatch(content_type: &str, extension: &str) -> bool {
    let expected = expected_extensions_for_content_type(content_type);
    !expected.is_empty() && !expected.contains(&extension)
}

fn expected_extensions_for_content_type(content_type: &str) -> &'static [&'static str] {
    match content_type {
        "application/pdf" => &["pdf"],
        "application/zip" => &["zip"],
        "image/jpeg" => &["jpg", "jpeg"],
        "image/png" => &["png"],
        "text/csv" => &["csv"],
        "text/plain" => &["txt", "text", "log", "csv"],
        _ => &[],
    }
}

fn max_scan_status(
    current: AttachmentSafetyScanStatus,
    candidate: AttachmentSafetyScanStatus,
) -> AttachmentSafetyScanStatus {
    if scan_status_rank(candidate) > scan_status_rank(current) {
        candidate
    } else {
        current
    }
}

fn scan_status_rank(status: AttachmentSafetyScanStatus) -> u8 {
    match status {
        AttachmentSafetyScanStatus::NotScanned => 0,
        AttachmentSafetyScanStatus::Clean => 1,
        AttachmentSafetyScanStatus::Suspicious => 2,
        AttachmentSafetyScanStatus::Failed => 3,
        AttachmentSafetyScanStatus::Malicious => 4,
    }
}

fn scan_summary(status: AttachmentSafetyScanStatus) -> &'static str {
    match status {
        AttachmentSafetyScanStatus::Malicious => "Executable payload detected",
        AttachmentSafetyScanStatus::Suspicious => "Attachment metadata requires safety review",
        AttachmentSafetyScanStatus::Failed => "Attachment safety scan failed",
        AttachmentSafetyScanStatus::Clean | AttachmentSafetyScanStatus::NotScanned => {
            "Attachment was not scanned by a safety backend"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(
        filename: Option<&'a str>,
        content_type: &'a str,
        bytes: &'a [u8],
    ) -> AttachmentSafetyScanRequest<'a> {
        AttachmentSafetyScanRequest {
            provider_attachment_id: "part-1",
            filename,
            content_type,
            size_bytes: bytes.len() as i64,
            sha256: "sha256:fixture",
            storage_kind: "local_fs",
            storage_path: "aa/bb/blob",
            bytes,
        }
    }

    #[test]
    fn heuristic_scanner_leaves_unmatched_attachments_not_scanned() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(Some("invoice.txt"), "text/plain", b"hello"))
            .expect("scan report");

        assert_eq!(report, AttachmentSafetyScanReport::not_scanned());
    }

    #[test]
    fn heuristic_scanner_marks_executable_payloads_malicious() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(
                Some("invoice.pdf"),
                "application/pdf",
                b"MZ\x90\x00fake portable executable",
            ))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Malicious);
        assert_eq!(report.engine.as_deref(), Some("hermes_heuristic_v1"));
        assert!(report.checked_at.is_some());
        assert_eq!(
            report.summary.as_deref(),
            Some("Executable payload detected")
        );
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["executable_magic"])
        );
    }

    #[test]
    fn heuristic_scanner_marks_mime_filename_mismatch_suspicious() {
        let scanner = HeuristicAttachmentSafetyScanner;
        let report = scanner
            .scan(&request(
                Some("invoice.pdf"),
                "application/zip",
                b"PK\x03\x04fake zip",
            ))
            .expect("scan report");

        assert_eq!(report.status, AttachmentSafetyScanStatus::Suspicious);
        assert_eq!(report.engine.as_deref(), Some("hermes_heuristic_v1"));
        assert!(report.checked_at.is_some());
        assert_eq!(
            report.summary.as_deref(),
            Some("Attachment metadata requires safety review")
        );
        assert_eq!(
            report.metadata["reasons"],
            serde_json::json!(["mime_extension_mismatch"])
        );
    }
}
