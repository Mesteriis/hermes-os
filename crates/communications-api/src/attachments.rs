use std::future::Future;
use std::pin::Pin;

use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Provider-neutral attachment metadata. The storage path is deliberately not
/// part of this contract; composition converts it into a scoped BlobRef.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AttachmentReference {
    pub attachment_id: String,
    pub blob_id: String,
    pub account_id: Option<String>,
    pub channel_kind: Option<String>,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub scan_status: Option<String>,
}

#[derive(Debug, Error)]
#[error("communication attachment lookup failed: {0}")]
pub struct AttachmentLookupPortError(pub String);

pub type AttachmentLookupPortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AttachmentLookupPortError>> + Send + 'a>>;

pub trait CommunicationAttachmentLookupPort: Send + Sync {
    fn lookup_by_id<'a>(
        &'a self,
        attachment_id: &'a str,
    ) -> AttachmentLookupPortFuture<'a, Option<AttachmentReference>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationMessageAttachmentRow {
    pub account_id: String,
    pub provider_record_id: String,
    pub message_metadata: serde_json::Value,
}

#[derive(Debug, Error)]
#[error("communication message attachment scan failed: {0}")]
pub struct CommunicationMessageAttachmentScanError(pub String);

pub type CommunicationMessageAttachmentScanFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    Vec<CommunicationMessageAttachmentRow>,
                    CommunicationMessageAttachmentScanError,
                >,
            > + Send
            + 'a,
    >,
>;

pub trait CommunicationMessageAttachmentScanPort: Send + Sync {
    fn scan_message_attachments<'a>(
        &'a self,
        account_id: &'a str,
        channel_kind: &'a str,
        limit: i64,
    ) -> CommunicationMessageAttachmentScanFuture<'a>;
}

#[derive(Clone, Debug)]
pub struct CanonicalMessageAttachmentRecord {
    pub message_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: String,
    pub attachment_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Error)]
#[error("canonical message attachment lookup failed: {0}")]
pub struct CanonicalMessageAttachmentReadError(pub String);

#[async_trait::async_trait]
pub trait CanonicalMessageAttachmentReadPort: Send + Sync {
    async fn list_for_messages(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<CanonicalMessageAttachmentRecord>, CanonicalMessageAttachmentReadError>;
}

#[derive(Clone, Debug)]
pub struct CanonicalMediaRecord {
    pub attachment_id: String,
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub channel_kind: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub scan_status: String,
    pub storage_kind: String,
    pub storage_path: String,
    pub message_subject: Option<String>,
    pub sender: Option<String>,
    pub sender_display_name: Option<String>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
#[error("canonical media lookup failed: {0}")]
pub struct CanonicalMediaReadError(pub String);

#[async_trait::async_trait]
pub trait CanonicalMediaReadPort: Send + Sync {
    async fn list_whatsapp_media(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        content_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CanonicalMediaRecord>, CanonicalMediaReadError>;
}
