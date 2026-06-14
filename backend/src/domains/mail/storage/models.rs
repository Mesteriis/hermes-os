use chrono::{DateTime, Utc};
use serde_json::Value;

use super::blob_store::LocalMailBlob;
use super::errors::MailStorageError;
use super::scanner::{AttachmentSafetyScanReport, AttachmentSafetyScanStatus};
use super::validation::{
    validate_non_empty, validate_sha256, validate_size_bytes, validate_storage_kind,
    validate_storage_path,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewMailBlob {
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
}

impl NewMailBlob {
    pub fn new(
        storage_kind: impl Into<String>,
        storage_path: impl Into<String>,
        sha256: impl Into<String>,
        size_bytes: i64,
    ) -> Self {
        Self {
            storage_kind: storage_kind.into(),
            storage_path: storage_path.into(),
            sha256: sha256.into(),
            size_bytes,
            content_type: None,
        }
    }

    pub fn from_local_blob(blob: &LocalMailBlob) -> Self {
        Self::new(
            &blob.storage_kind,
            &blob.storage_path,
            &blob.sha256,
            blob.size_bytes,
        )
    }

    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<ValidatedMailBlob, MailStorageError> {
        let storage_kind = validate_storage_kind(&self.storage_kind)?;
        let storage_path = validate_storage_path(&self.storage_path)?;
        let sha256 = validate_sha256(&self.sha256)?;
        let size_bytes = validate_size_bytes(self.size_bytes)?;
        let content_type = self
            .content_type
            .as_deref()
            .map(|value| validate_non_empty("content_type", value))
            .transpose()?;

        Ok(ValidatedMailBlob {
            storage_kind,
            storage_path,
            sha256,
            size_bytes,
            content_type,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedMailBlob {
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
    pub(crate) content_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredMailBlob {
    pub blob_id: String,
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewMailAttachment {
    pub message_id: String,
    pub raw_record_id: String,
    pub blob_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub disposition: MailAttachmentDisposition,
    pub scan_report: AttachmentSafetyScanReport,
}

impl NewMailAttachment {
    pub fn new(
        message_id: impl Into<String>,
        raw_record_id: impl Into<String>,
        blob_id: impl Into<String>,
        provider_attachment_id: impl Into<String>,
        content_type: impl Into<String>,
        size_bytes: i64,
        sha256: impl Into<String>,
    ) -> Self {
        Self {
            message_id: message_id.into(),
            raw_record_id: raw_record_id.into(),
            blob_id: blob_id.into(),
            provider_attachment_id: provider_attachment_id.into(),
            filename: None,
            content_type: content_type.into(),
            size_bytes,
            sha256: sha256.into(),
            disposition: MailAttachmentDisposition::Unknown,
            scan_report: AttachmentSafetyScanReport::not_scanned(),
        }
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn disposition(mut self, disposition: MailAttachmentDisposition) -> Self {
        self.disposition = disposition;
        self
    }

    pub fn scan_report(mut self, scan_report: AttachmentSafetyScanReport) -> Self {
        self.scan_report = scan_report;
        self
    }

    pub(crate) fn validate(&self) -> Result<ValidatedMailAttachment, MailStorageError> {
        let message_id = validate_non_empty("message_id", &self.message_id)?;
        let raw_record_id = validate_non_empty("raw_record_id", &self.raw_record_id)?;
        let blob_id = validate_non_empty("blob_id", &self.blob_id)?;
        let provider_attachment_id =
            validate_non_empty("provider_attachment_id", &self.provider_attachment_id)?;
        let filename = self
            .filename
            .as_deref()
            .map(|value| validate_non_empty("filename", value))
            .transpose()?;
        let content_type = validate_non_empty("content_type", &self.content_type)?;
        let size_bytes = validate_size_bytes(self.size_bytes)?;
        let sha256 = validate_sha256(&self.sha256)?;
        let scan_report = self.scan_report.validate()?;

        Ok(ValidatedMailAttachment {
            message_id,
            raw_record_id,
            blob_id,
            provider_attachment_id,
            filename,
            content_type,
            size_bytes,
            sha256,
            disposition: self.disposition,
            scan_report,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ValidatedMailAttachment {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) blob_id: String,
    pub(crate) provider_attachment_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) disposition: MailAttachmentDisposition,
    pub(crate) scan_report: AttachmentSafetyScanReport,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredMailAttachment {
    pub attachment_id: String,
    pub message_id: String,
    pub raw_record_id: String,
    pub blob_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub disposition: MailAttachmentDisposition,
    pub scan_status: AttachmentSafetyScanStatus,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
    pub scan_metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredMailAttachmentWithBlob {
    pub attachment: StoredMailAttachment,
    pub storage_kind: String,
    pub storage_path: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAttachmentDisposition {
    Attachment,
    Inline,
    Unknown,
}

impl MailAttachmentDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attachment => "attachment",
            Self::Inline => "inline",
            Self::Unknown => "unknown",
        }
    }
}

impl TryFrom<&str> for MailAttachmentDisposition {
    type Error = MailStorageError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "attachment" => Ok(Self::Attachment),
            "inline" => Ok(Self::Inline),
            "unknown" => Ok(Self::Unknown),
            other => Err(MailStorageError::InvalidDisposition(other.to_owned())),
        }
    }
}
