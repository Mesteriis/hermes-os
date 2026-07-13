use crate::domains::communications::messages::port::MessageProjectionPort;
use crate::domains::communications::storage::port::{CommunicationAttachmentPort, LocalBlobPort};
use std::path::Path;

use serde_json::{Map, Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::messages::MessageProjectionError;
use crate::domains::communications::storage::{
    AttachmentSafetyScanError, AttachmentSafetyScanRequest, AttachmentSafetyScanner,
    CommunicationAttachmentDisposition, CommunicationStorageError,
    HeuristicAttachmentSafetyScanner, NewCommunicationAttachment, NewCommunicationBlob,
};
use crate::platform::communications::{
    ProviderChannelMessageLookupPort, ProviderCommunicationMessagePortError,
};

const ZULIP_CHANNEL_KIND: &str = "zulip";
const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipAttachmentBytes {
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZulipAttachmentMaterialization {
    pub message_id: String,
    pub raw_record_id: String,
    pub provider_message_id: String,
    pub provider_attachment_id: String,
    pub attachment_id: String,
    pub blob_id: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub scan_status: String,
    pub message_metadata: Value,
}

pub async fn persist_zulip_attachment_bytes(
    pool: PgPool,
    message_lookup: &impl ProviderChannelMessageLookupPort,
    request: &ZulipAttachmentBytes,
    blob_root: &Path,
) -> Result<ZulipAttachmentMaterialization, ZulipAttachmentStorageError> {
    let account_id = trim_required("account_id", &request.account_id)?;
    let provider_message_id = trim_required("provider_message_id", &request.provider_message_id)?;
    let provider_attachment_id =
        trim_required("provider_attachment_id", &request.provider_attachment_id)?;
    let filename = request
        .filename
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let content_type = request
        .content_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_CONTENT_TYPE.to_owned());

    let message = message_lookup
        .message_by_provider_record_id(account_id, provider_message_id, &[ZULIP_CHANNEL_KIND])
        .await?
        .ok_or_else(|| ZulipAttachmentStorageError::MessageAnchorNotFound {
            account_id: account_id.to_owned(),
            provider_message_id: provider_message_id.to_owned(),
        })?;

    let blob_store = LocalBlobPort::new(blob_root);
    let local_blob = blob_store.put_blob(&request.bytes).await?;
    let metadata_store = CommunicationAttachmentPort::new(pool.clone());
    let stored_blob = metadata_store
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&local_blob).content_type(&content_type),
        )
        .await?;

    let scanner = HeuristicAttachmentSafetyScanner;
    let scan_report = scanner.scan(&AttachmentSafetyScanRequest {
        provider_attachment_id,
        filename: filename.as_deref(),
        content_type: &content_type,
        size_bytes: local_blob.size_bytes,
        sha256: &stored_blob.sha256,
        storage_kind: &stored_blob.storage_kind,
        storage_path: &stored_blob.storage_path,
        bytes: &request.bytes,
    })?;
    let scan_status = scan_report.status.as_str().to_owned();

    let mut attachment = NewCommunicationAttachment::new(
        &message.message_id,
        &message.raw_record_id,
        &stored_blob.blob_id,
        provider_attachment_id,
        &content_type,
        local_blob.size_bytes,
        &stored_blob.sha256,
    )
    .disposition(CommunicationAttachmentDisposition::Attachment)
    .scan_report(scan_report);
    if let Some(filename) = filename.as_deref() {
        attachment = attachment.filename(filename);
    }
    let stored_attachment = metadata_store.upsert_attachment(&attachment).await?;

    let updated_metadata = materialized_message_metadata(
        &message.message_metadata,
        &MaterializedAttachmentMetadata {
            attachment_id: &stored_attachment.attachment_id,
            blob_id: &stored_blob.blob_id,
            provider_attachment_id,
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &stored_blob.sha256,
            scan_status: &scan_status,
        },
    )?;
    let projected = MessageProjectionPort::new(pool)
        .set_message_metadata(&message.message_id, &updated_metadata)
        .await?;

    Ok(ZulipAttachmentMaterialization {
        message_id: message.message_id,
        raw_record_id: message.raw_record_id,
        provider_message_id: provider_message_id.to_owned(),
        provider_attachment_id: provider_attachment_id.to_owned(),
        attachment_id: stored_attachment.attachment_id,
        blob_id: stored_blob.blob_id,
        content_type,
        size_bytes: local_blob.size_bytes,
        sha256: stored_blob.sha256,
        scan_status,
        message_metadata: projected.message_metadata,
    })
}

fn materialized_message_metadata(
    metadata: &Value,
    materialized: &MaterializedAttachmentMetadata<'_>,
) -> Result<Value, ZulipAttachmentStorageError> {
    let mut metadata_object = metadata.as_object().cloned().unwrap_or_default();
    let attachment_state = {
        let attachments = metadata_object
            .entry("attachments".to_owned())
            .or_insert_with(|| Value::Array(Vec::new()));
        let attachment_array = attachments.as_array_mut().ok_or(
            ZulipAttachmentStorageError::InvalidMessageMetadata {
                reason: "attachments metadata must be an array",
            },
        )?;

        let mut matched = false;
        for attachment in attachment_array.iter_mut() {
            let Some(object) = attachment.as_object_mut() else {
                continue;
            };
            if !matches_provider_attachment_id(object, materialized.provider_attachment_id) {
                continue;
            }
            apply_materialized_attachment_metadata(object, materialized);
            matched = true;
        }

        if !matched {
            let mut object = Map::new();
            apply_materialized_attachment_metadata(&mut object, materialized);
            attachment_array.push(Value::Object(object));
        }

        attachment_state_metadata(attachment_array)
    };

    metadata_object.insert("attachment_state".to_owned(), attachment_state);

    Ok(Value::Object(metadata_object))
}

struct MaterializedAttachmentMetadata<'a> {
    attachment_id: &'a str,
    blob_id: &'a str,
    provider_attachment_id: &'a str,
    filename: Option<&'a str>,
    content_type: &'a str,
    size_bytes: i64,
    sha256: &'a str,
    scan_status: &'a str,
}

fn apply_materialized_attachment_metadata(
    object: &mut Map<String, Value>,
    materialized: &MaterializedAttachmentMetadata<'_>,
) {
    object.insert("provider".to_owned(), json!("zulip"));
    object.insert(
        "attachment_id".to_owned(),
        json!(materialized.attachment_id),
    );
    object.insert(
        "provider_attachment_id".to_owned(),
        json!(materialized.provider_attachment_id),
    );
    object.insert("blob_id".to_owned(), json!(materialized.blob_id));
    object.insert("bytes_state".to_owned(), json!("transferred"));
    object.insert("materialization_state".to_owned(), json!("materialized"));
    object.insert("scan_status".to_owned(), json!(materialized.scan_status));
    object.insert("content_type".to_owned(), json!(materialized.content_type));
    object.insert("size_bytes".to_owned(), json!(materialized.size_bytes));
    object.insert("sha256".to_owned(), json!(materialized.sha256));
    if let Some(filename) = materialized.filename {
        object.insert("filename".to_owned(), json!(filename));
    }
}

fn matches_provider_attachment_id(
    object: &Map<String, Value>,
    provider_attachment_id: &str,
) -> bool {
    object
        .get("provider_attachment_id")
        .or_else(|| object.get("attachment_id"))
        .and_then(Value::as_str)
        .map(|value| value == provider_attachment_id)
        .unwrap_or(false)
}

fn attachment_state_metadata(attachments: &[Value]) -> Value {
    let attachment_count = attachments.len();
    let materialized_count = attachments
        .iter()
        .filter(|attachment| {
            attachment
                .get("materialization_state")
                .and_then(Value::as_str)
                == Some("materialized")
        })
        .count();
    let materialization_state = if materialized_count == attachment_count {
        "materialized"
    } else if materialized_count == 0 {
        "not_materialized"
    } else {
        "partially_materialized"
    };
    let bytes_state = if materialized_count == attachment_count {
        "transferred"
    } else if materialized_count == 0 {
        "not_transferred"
    } else {
        "partially_transferred"
    };

    json!({
        "state": materialization_state,
        "bytes_state": bytes_state,
        "materialization_state": materialization_state,
        "scan_status": strongest_scan_status(attachments),
        "attachment_count": attachment_count,
        "materialized_count": materialized_count,
    })
}

fn strongest_scan_status(attachments: &[Value]) -> &'static str {
    let status = attachments
        .iter()
        .filter_map(|attachment| attachment.get("scan_status").and_then(Value::as_str))
        .max_by_key(|status| scan_status_rank(status))
        .unwrap_or("not_scanned");
    canonical_scan_status(status)
}

fn scan_status_rank(status: &str) -> u8 {
    match status {
        "clean" => 1,
        "suspicious" => 2,
        "failed" => 3,
        "malicious" => 4,
        _ => 0,
    }
}

fn canonical_scan_status(status: &str) -> &'static str {
    match status {
        "clean" => "clean",
        "suspicious" => "suspicious",
        "failed" => "failed",
        "malicious" => "malicious",
        _ => "not_scanned",
    }
}

fn trim_required<'a>(
    field: &'static str,
    value: &'a str,
) -> Result<&'a str, ZulipAttachmentStorageError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ZulipAttachmentStorageError::InvalidRequest {
            field,
            reason: "must not be empty",
        });
    }
    Ok(value)
}

#[derive(Debug, Error)]
pub enum ZulipAttachmentStorageError {
    #[error("invalid Zulip attachment request field `{field}`: {reason}")]
    InvalidRequest {
        field: &'static str,
        reason: &'static str,
    },
    #[error(
        "Zulip message anchor was not found for account `{account_id}` message `{provider_message_id}`"
    )]
    MessageAnchorNotFound {
        account_id: String,
        provider_message_id: String,
    },
    #[error("invalid Zulip message metadata: {reason}")]
    InvalidMessageMetadata { reason: &'static str },
    #[error(transparent)]
    ProviderMessage(#[from] ProviderCommunicationMessagePortError),
    #[error(transparent)]
    Storage(#[from] CommunicationStorageError),
    #[error(transparent)]
    Scan(#[from] AttachmentSafetyScanError),
    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),
}
