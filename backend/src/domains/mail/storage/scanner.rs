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
pub struct NoopAttachmentSafetyScanner;

impl AttachmentSafetyScanner for NoopAttachmentSafetyScanner {
    fn scan(
        &self,
        _request: &AttachmentSafetyScanRequest<'_>,
    ) -> Result<AttachmentSafetyScanReport, AttachmentSafetyScanError> {
        Ok(AttachmentSafetyScanReport::not_scanned())
    }
}
