use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::domains::mail::storage::{AttachmentSafetyScanStatus, MailAttachmentDisposition};

#[derive(Clone, Debug)]
pub struct AttachmentSearchQuery<'a> {
    pub account_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub content_type: Option<&'a str>,
    pub scan_status: Option<&'a str>,
    pub cursor: Option<&'a str>,
    pub limit: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentSearchPage {
    pub items: Vec<AttachmentSearchResult>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentSearchResult {
    pub attachment_id: String,
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub message_subject: String,
    pub sender: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub blob_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub disposition: MailAttachmentDispositionDto,
    pub scan_status: AttachmentSafetyScanStatusDto,
    pub scan_engine: Option<String>,
    pub scan_checked_at: Option<DateTime<Utc>>,
    pub scan_summary: Option<String>,
    pub storage_kind: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MailAttachmentDispositionDto {
    Attachment,
    Inline,
    Unknown,
}

impl From<MailAttachmentDisposition> for MailAttachmentDispositionDto {
    fn from(value: MailAttachmentDisposition) -> Self {
        match value {
            MailAttachmentDisposition::Attachment => Self::Attachment,
            MailAttachmentDisposition::Inline => Self::Inline,
            MailAttachmentDisposition::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentSafetyScanStatusDto {
    NotScanned,
    Clean,
    Suspicious,
    Malicious,
    Failed,
}

impl From<AttachmentSafetyScanStatus> for AttachmentSafetyScanStatusDto {
    fn from(value: AttachmentSafetyScanStatus) -> Self {
        match value {
            AttachmentSafetyScanStatus::NotScanned => Self::NotScanned,
            AttachmentSafetyScanStatus::Clean => Self::Clean,
            AttachmentSafetyScanStatus::Suspicious => Self::Suspicious,
            AttachmentSafetyScanStatus::Malicious => Self::Malicious,
            AttachmentSafetyScanStatus::Failed => Self::Failed,
        }
    }
}

#[derive(Clone)]
pub struct AttachmentSearchStore {
    pool: PgPool,
}

impl AttachmentSearchStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search(
        &self,
        request: AttachmentSearchQuery<'_>,
    ) -> Result<AttachmentSearchPage, AttachmentSearchError> {
        let account_id = normalize_optional(request.account_id);
        let query = normalize_optional(request.query);
        let content_type = normalize_optional(request.content_type);
        let scan_status = normalize_optional(request.scan_status)
            .map(validate_scan_status)
            .transpose()?;
        let cursor = normalize_optional(request.cursor)
            .map(decode_attachment_search_cursor)
            .transpose()?;
        let cursor_created_at = cursor.as_ref().map(|cursor| cursor.created_at);
        let cursor_attachment_id = cursor.as_ref().map(|cursor| cursor.attachment_id.as_str());
        let limit = request.limit.clamp(1, 250);
        let fetch_limit = limit + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
                m.account_id,
                m.subject AS message_subject,
                m.sender,
                m.occurred_at,
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
                b.storage_kind,
                b.storage_path,
                a.created_at,
                a.updated_at
            FROM communication_attachments a
            JOIN communication_messages m ON m.message_id = a.message_id
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE m.local_state = 'active'
              AND ($1::text IS NULL OR m.account_id = $1)
              AND ($2::text IS NULL OR a.content_type ILIKE '%' || $2 || '%')
              AND ($3::text IS NULL OR a.scan_status = $3)
              AND (
                $4::text IS NULL
                OR NOT EXISTS (
                  SELECT 1
                  FROM unnest(regexp_split_to_array(lower(trim($4)), '\s+')) AS term
                  WHERE term <> ''
                    AND lower(
                      concat_ws(
                        ' ',
                        a.filename,
                        a.content_type,
                        a.sha256,
                        a.provider_attachment_id,
                        m.subject,
                        m.sender
                      )
                    ) NOT LIKE '%' || term || '%'
                )
              )
              AND (
                $5::timestamptz IS NULL
                OR a.created_at < $5
                OR (a.created_at = $5 AND a.attachment_id > $6)
              )
            ORDER BY a.created_at DESC, a.attachment_id ASC
            LIMIT $7
            "#,
        )
        .bind(account_id)
        .bind(content_type)
        .bind(scan_status)
        .bind(query)
        .bind(cursor_created_at)
        .bind(cursor_attachment_id)
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;

        let has_more = rows.len() > limit as usize;
        let items = rows
            .into_iter()
            .take(limit as usize)
            .map(row_to_attachment_search_result)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = if has_more {
            items
                .last()
                .map(encode_attachment_search_cursor)
                .transpose()?
        } else {
            None
        };

        Ok(AttachmentSearchPage {
            items,
            next_cursor,
            has_more,
        })
    }
}

fn row_to_attachment_search_result(
    row: PgRow,
) -> Result<AttachmentSearchResult, AttachmentSearchError> {
    let disposition: String = row.try_get("disposition")?;
    let scan_status: String = row.try_get("scan_status")?;
    Ok(AttachmentSearchResult {
        attachment_id: row.try_get("attachment_id")?,
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        message_subject: row.try_get("message_subject")?,
        sender: row.try_get("sender")?,
        occurred_at: row.try_get("occurred_at")?,
        blob_id: row.try_get("blob_id")?,
        provider_attachment_id: row.try_get("provider_attachment_id")?,
        filename: row.try_get("filename")?,
        content_type: row.try_get("content_type")?,
        size_bytes: row.try_get("size_bytes")?,
        sha256: row.try_get("sha256")?,
        disposition: MailAttachmentDisposition::try_from(disposition.as_str())?.into(),
        scan_status: AttachmentSafetyScanStatus::try_from(scan_status.as_str())?.into(),
        scan_engine: row.try_get("scan_engine")?,
        scan_checked_at: row.try_get("scan_checked_at")?,
        scan_summary: row.try_get("scan_summary")?,
        storage_kind: row.try_get("storage_kind")?,
        storage_path: row.try_get("storage_path")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Deserialize, Serialize)]
struct AttachmentSearchCursor {
    created_at: DateTime<Utc>,
    attachment_id: String,
}

fn encode_attachment_search_cursor(
    item: &AttachmentSearchResult,
) -> Result<String, AttachmentSearchError> {
    let cursor = AttachmentSearchCursor {
        created_at: item.created_at,
        attachment_id: item.attachment_id.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| AttachmentSearchError::InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_attachment_search_cursor(
    cursor: &str,
) -> Result<AttachmentSearchCursor, AttachmentSearchError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AttachmentSearchError::InvalidCursor)?;
    let cursor: AttachmentSearchCursor =
        serde_json::from_slice(&bytes).map_err(|_| AttachmentSearchError::InvalidCursor)?;
    if cursor.attachment_id.trim().is_empty() {
        return Err(AttachmentSearchError::InvalidCursor);
    }
    Ok(cursor)
}

fn normalize_optional(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn validate_scan_status(value: &str) -> Result<&str, AttachmentSearchError> {
    AttachmentSafetyScanStatus::try_from(value)?;
    Ok(value)
}

#[derive(Debug, Error)]
pub enum AttachmentSearchError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    MailStorage(#[from] crate::domains::mail::storage::MailStorageError),
    #[error("invalid attachment search cursor")]
    InvalidCursor,
}
