use super::super::*;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use crate::domains::mail::storage::{
    AttachmentSafetyScanRequest, AttachmentSafetyScanner, HeuristicAttachmentSafetyScanner,
    NewCommunicationAttachmentImport, NewMailBlob, new_communication_attachment_import_id,
};

const MAX_ATTACHMENT_IMPORT_BYTES: usize = 50 * 1024 * 1024;
const LOCAL_IMPORT_ACTOR_ID: &str = "hermes-frontend";

#[derive(Deserialize)]
pub(crate) struct CommunicationAttachmentImportRequest {
    pub(crate) account_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) content_base64: String,
    pub(crate) source_kind: Option<String>,
    pub(crate) metadata: Option<Value>,
}

#[derive(Serialize)]
pub(crate) struct CommunicationAttachmentImportResponse {
    pub(crate) attachment_id: String,
    pub(crate) account_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) blob_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) scan_status: String,
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
}

pub(crate) async fn post_v1_attachment_import(
    State(state): State<AppState>,
    Json(request): Json<CommunicationAttachmentImportRequest>,
) -> Result<Json<CommunicationAttachmentImportResponse>, ApiError> {
    let bytes = decode_import_bytes(&request.content_base64)?;
    if bytes.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment import bytes must not be empty",
        ));
    }
    if bytes.len() > MAX_ATTACHMENT_IMPORT_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment import exceeds the local size limit",
        ));
    }

    let content_type = request
        .content_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("application/octet-stream")
        .to_owned();
    let filename = request
        .filename
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let metadata = request.metadata.unwrap_or_else(|| json!({}));
    if !metadata.is_object() {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment import metadata must be an object",
        ));
    }

    let blob_store = LocalMailBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = blob_store.put_blob(&bytes).await?;
    let mail_store = mail_storage_store(&state)?;
    let stored_blob = mail_store
        .upsert_blob(&NewMailBlob::from_local_blob(&local_blob).content_type(&content_type))
        .await?;
    let scanner = HeuristicAttachmentSafetyScanner;
    let scan_report = scanner
        .scan(&AttachmentSafetyScanRequest {
            provider_attachment_id: "local-import",
            filename: filename.as_deref(),
            content_type: &content_type,
            size_bytes: local_blob.size_bytes,
            sha256: &local_blob.sha256,
            storage_kind: &local_blob.storage_kind,
            storage_path: &local_blob.storage_path,
            bytes: &bytes,
        })
        .map_err(|error| {
            tracing::warn!(error = %error, "communication attachment import safety scan failed");
            ApiError::InvalidCommunicationQuery("attachment safety scan failed")
        })?;
    let seed = format!(
        "{}:{}:{}:{}",
        local_blob.sha256,
        filename.as_deref().unwrap_or(""),
        request.account_id.as_deref().unwrap_or(""),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let mut import = NewCommunicationAttachmentImport::new(
        new_communication_attachment_import_id(&seed),
        stored_blob.blob_id,
        &content_type,
        local_blob.size_bytes,
        &local_blob.sha256,
        LOCAL_IMPORT_ACTOR_ID,
    )
    .source_kind(
        request
            .source_kind
            .unwrap_or_else(|| "local_import".to_owned()),
    )
    .scan_report(scan_report)
    .metadata(metadata);
    if let Some(account_id) = request.account_id {
        import = import.account_id(account_id);
    }
    if let Some(channel_kind) = request.channel_kind {
        import = import.channel_kind(channel_kind);
    }
    if let Some(filename) = filename {
        import = import.filename(filename);
    }

    let imported = mail_store.upsert_imported_attachment(&import).await?;
    Ok(Json(CommunicationAttachmentImportResponse {
        attachment_id: imported.attachment_id,
        account_id: imported.account_id,
        channel_kind: imported.channel_kind,
        blob_id: imported.blob_id,
        filename: imported.filename,
        content_type: imported.content_type,
        size_bytes: imported.size_bytes,
        sha256: imported.sha256,
        scan_status: imported.scan_status.as_str().to_owned(),
        storage_kind: imported.storage_kind,
        storage_path: imported.storage_path,
    }))
}

fn decode_import_bytes(content_base64: &str) -> Result<Vec<u8>, ApiError> {
    let encoded = content_base64
        .split_once(',')
        .map(|(_, value)| value)
        .unwrap_or(content_base64)
        .trim();
    BASE64_STANDARD
        .decode(encoded)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid attachment import base64"))
}
