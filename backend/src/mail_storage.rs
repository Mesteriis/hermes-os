use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

const LOCAL_FS_STORAGE_KIND: &str = "local_fs";
const SHA256_PREFIX: &str = "sha256:";

#[derive(Clone, Debug)]
pub struct LocalMailBlobStore {
    root: PathBuf,
}

impl LocalMailBlobStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub async fn put_blob(&self, bytes: &[u8]) -> Result<LocalMailBlob, MailStorageError> {
        let size_bytes = i64::try_from(bytes.len()).map_err(|_| MailStorageError::BlobTooLarge)?;
        let digest_hex = sha256_hex(bytes);
        let storage_path = relative_blob_path(&digest_hex);
        let absolute_path = self.root.join(&storage_path);

        if let Some(parent) = absolute_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        if !path_exists(&absolute_path).await? {
            let temp_path = absolute_path.with_extension(format!(
                "tmp-{}-{}",
                std::process::id(),
                Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ));
            tokio::fs::write(&temp_path, bytes).await?;
            tokio::fs::rename(&temp_path, &absolute_path).await?;
        }

        let metadata = tokio::fs::metadata(&absolute_path).await?;
        let actual_size =
            i64::try_from(metadata.len()).map_err(|_| MailStorageError::BlobTooLarge)?;
        if actual_size != size_bytes {
            return Err(MailStorageError::BlobSizeMismatch {
                path: absolute_path,
                expected: size_bytes,
                actual: actual_size,
            });
        }

        Ok(LocalMailBlob {
            storage_kind: LOCAL_FS_STORAGE_KIND.to_owned(),
            storage_path,
            sha256: format!("{SHA256_PREFIX}{digest_hex}"),
            size_bytes,
        })
    }

    pub async fn read_blob(&self, storage_path: &str) -> Result<Vec<u8>, MailStorageError> {
        let storage_path = validate_storage_path(storage_path)?;
        Ok(tokio::fs::read(self.root.join(storage_path)).await?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalMailBlob {
    pub storage_kind: String,
    pub storage_path: String,
    pub sha256: String,
    pub size_bytes: i64,
}

#[derive(Clone)]
pub struct MailStorageStore {
    pool: PgPool,
}

impl MailStorageStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_blob(
        &self,
        blob: &NewMailBlob,
    ) -> Result<StoredMailBlob, MailStorageError> {
        let blob = blob.validate()?;
        let blob_id = mail_blob_id(&blob.sha256);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_mail_blobs (
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (sha256)
            DO UPDATE SET
                content_type = COALESCE(communication_mail_blobs.content_type, EXCLUDED.content_type)
            RETURNING
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type,
                created_at
            "#,
        )
        .bind(&blob_id)
        .bind(&blob.storage_kind)
        .bind(&blob.storage_path)
        .bind(&blob.sha256)
        .bind(blob.size_bytes)
        .bind(&blob.content_type)
        .fetch_one(&self.pool)
        .await?;

        row_to_mail_blob(row)
    }

    pub async fn upsert_attachment(
        &self,
        attachment: &NewMailAttachment,
    ) -> Result<StoredMailAttachment, MailStorageError> {
        let attachment = attachment.validate()?;
        let attachment_id =
            mail_attachment_id(&attachment.message_id, &attachment.provider_attachment_id);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_attachments (
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
            ON CONFLICT (message_id, provider_attachment_id)
            DO UPDATE SET
                raw_record_id = EXCLUDED.raw_record_id,
                blob_id = EXCLUDED.blob_id,
                filename = EXCLUDED.filename,
                content_type = EXCLUDED.content_type,
                size_bytes = EXCLUDED.size_bytes,
                sha256 = EXCLUDED.sha256,
                disposition = EXCLUDED.disposition,
                scan_status = EXCLUDED.scan_status,
                scan_engine = EXCLUDED.scan_engine,
                scan_checked_at = EXCLUDED.scan_checked_at,
                scan_summary = EXCLUDED.scan_summary,
                scan_metadata = EXCLUDED.scan_metadata,
                updated_at = now()
            RETURNING
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&attachment_id)
        .bind(&attachment.message_id)
        .bind(&attachment.raw_record_id)
        .bind(&attachment.blob_id)
        .bind(&attachment.provider_attachment_id)
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size_bytes)
        .bind(&attachment.sha256)
        .bind(attachment.disposition.as_str())
        .bind(attachment.scan_report.status.as_str())
        .bind(&attachment.scan_report.engine)
        .bind(attachment.scan_report.checked_at)
        .bind(&attachment.scan_report.summary)
        .bind(&attachment.scan_report.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_mail_attachment(row)
    }

    pub async fn attachments_for_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<StoredMailAttachmentWithBlob>, MailStorageError> {
        let message_id = validate_non_empty("message_id", message_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
                a.blob_id,
                a.provider_attachment_id,
                a.filename,
                a.content_type,
                a.size_bytes,
                a.sha256,
                a.disposition,
                a.scan_status,
                a.scan_engine,
                a.scan_checked_at,
                a.scan_summary,
                a.scan_metadata,
                a.created_at,
                a.updated_at,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path
            FROM communication_attachments a
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE a.message_id = $1
            ORDER BY a.created_at ASC, a.attachment_id ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_mail_attachment_with_blob)
            .collect()
    }
}

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

    fn validate(&self) -> Result<ValidatedMailBlob, MailStorageError> {
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
struct ValidatedMailBlob {
    storage_kind: String,
    storage_path: String,
    sha256: String,
    size_bytes: i64,
    content_type: Option<String>,
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

    fn validate(&self) -> Result<ValidatedMailAttachment, MailStorageError> {
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
struct ValidatedMailAttachment {
    message_id: String,
    raw_record_id: String,
    blob_id: String,
    provider_attachment_id: String,
    filename: Option<String>,
    content_type: String,
    size_bytes: i64,
    sha256: String,
    disposition: MailAttachmentDisposition,
    scan_report: AttachmentSafetyScanReport,
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

    fn validate(&self) -> Result<Self, MailStorageError> {
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

#[derive(Debug, Error)]
pub enum AttachmentSafetyScanError {
    #[error("attachment safety scanner failed: {0}")]
    Scanner(String),
}

fn row_to_mail_blob(row: PgRow) -> Result<StoredMailBlob, MailStorageError> {
    Ok(StoredMailBlob {
        blob_id: row.try_get("blob_id")?,
        storage_kind: row.try_get("storage_kind")?,
        storage_path: row.try_get("storage_path")?,
        sha256: row.try_get("sha256")?,
        size_bytes: row.try_get("size_bytes")?,
        content_type: row.try_get("content_type")?,
        created_at: row.try_get("created_at")?,
    })
}

fn row_to_mail_attachment(row: PgRow) -> Result<StoredMailAttachment, MailStorageError> {
    let disposition: String = row.try_get("disposition")?;
    let scan_status: String = row.try_get("scan_status")?;

    Ok(StoredMailAttachment {
        attachment_id: row.try_get("attachment_id")?,
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        blob_id: row.try_get("blob_id")?,
        provider_attachment_id: row.try_get("provider_attachment_id")?,
        filename: row.try_get("filename")?,
        content_type: row.try_get("content_type")?,
        size_bytes: row.try_get("size_bytes")?,
        sha256: row.try_get("sha256")?,
        disposition: MailAttachmentDisposition::try_from(disposition.as_str())?,
        scan_status: AttachmentSafetyScanStatus::try_from(scan_status.as_str())?,
        scan_engine: row.try_get("scan_engine")?,
        scan_checked_at: row.try_get("scan_checked_at")?,
        scan_summary: row.try_get("scan_summary")?,
        scan_metadata: row.try_get("scan_metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_mail_attachment_with_blob(
    row: PgRow,
) -> Result<StoredMailAttachmentWithBlob, MailStorageError> {
    let storage_kind: String = row.try_get("blob_storage_kind")?;
    let storage_path: String = row.try_get("blob_storage_path")?;
    Ok(StoredMailAttachmentWithBlob {
        attachment: row_to_mail_attachment(row)?,
        storage_kind,
        storage_path,
    })
}

async fn path_exists(path: &Path) -> Result<bool, std::io::Error> {
    match tokio::fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn relative_blob_path(digest_hex: &str) -> String {
    format!("sha256/{}/{}.blob", &digest_hex[..2], digest_hex)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(hex_char(byte >> 4));
        encoded.push(hex_char(byte & 0x0f));
    }
    encoded
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex nibble must fit in 0..=15"),
    }
}

fn mail_blob_id(sha256: &str) -> String {
    format!("blob:v1:{sha256}")
}

fn mail_attachment_id(message_id: &str, provider_attachment_id: &str) -> String {
    let mut encoded = String::from("att:v1:");
    append_id_component(&mut encoded, message_id);
    encoded.push(':');
    append_id_component(&mut encoded, provider_attachment_id);
    encoded
}

fn append_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}

fn validate_storage_kind(value: &str) -> Result<String, MailStorageError> {
    let value = validate_non_empty("storage_kind", value)?;
    if value != LOCAL_FS_STORAGE_KIND {
        return Err(MailStorageError::InvalidStorageKind(value));
    }
    Ok(value)
}

fn validate_storage_path(value: &str) -> Result<String, MailStorageError> {
    let value = validate_non_empty("storage_path", value)?;
    let path = Path::new(&value);
    if path.is_absolute() || value.contains('\\') {
        return Err(MailStorageError::UnsafeStoragePath(value));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(MailStorageError::UnsafeStoragePath(value)),
        }
    }

    Ok(value)
}

fn validate_sha256(value: &str) -> Result<String, MailStorageError> {
    let value = validate_non_empty("sha256", value)?;
    let Some(hex) = value.strip_prefix(SHA256_PREFIX) else {
        return Err(MailStorageError::InvalidSha256(value));
    };
    if hex.len() != 64 || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(MailStorageError::InvalidSha256(value));
    }
    Ok(format!("{SHA256_PREFIX}{}", hex.to_ascii_lowercase()))
}

fn validate_size_bytes(value: i64) -> Result<i64, MailStorageError> {
    if value < 0 {
        return Err(MailStorageError::NegativeSizeBytes(value));
    }
    Ok(value)
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<String, MailStorageError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(MailStorageError::EmptyField(field_name));
    }
    Ok(value)
}

#[derive(Debug, Error)]
pub enum MailStorageError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("storage_kind must be local_fs: {0}")]
    InvalidStorageKind(String),

    #[error("storage_path must be relative and stay inside mail blob root: {0}")]
    UnsafeStoragePath(String),

    #[error("sha256 must use sha256:<64 lowercase hex chars>: {0}")]
    InvalidSha256(String),

    #[error("size_bytes must not be negative: {0}")]
    NegativeSizeBytes(i64),

    #[error("blob content is too large to represent as i64 size_bytes")]
    BlobTooLarge,

    #[error("blob size mismatch for {path}: expected {expected}, actual {actual}")]
    BlobSizeMismatch {
        path: PathBuf,
        expected: i64,
        actual: i64,
    },

    #[error("invalid attachment disposition: {0}")]
    InvalidDisposition(String),

    #[error("invalid attachment scan status: {0}")]
    InvalidScanStatus(String),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),

    #[error("not_scanned attachment scan reports must not include engine, checked_at or summary")]
    InvalidNotScannedReport,
}
