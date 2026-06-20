use super::super::*;
use crate::domains::communications::archive_inspection::{
    ArchiveInspectionLimits, ArchiveInspectionReport, inspect_zip_bytes,
};
use crate::domains::communications::attachment_search::{
    AttachmentSearchQuery, AttachmentSearchStore,
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

const MAX_TEXT_PREVIEW_BYTES: usize = 64 * 1024;
const MAX_IMAGE_PREVIEW_BYTES: usize = 5 * 1024 * 1024;

#[derive(Deserialize)]
pub(crate) struct AttachmentSearchRequest {
    pub(crate) account_id: Option<String>,
    pub(crate) q: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) scan_status: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

pub(crate) async fn get_v1_attachment_search(
    State(state): State<AppState>,
    Query(query): Query<AttachmentSearchRequest>,
) -> Result<Json<crate::domains::communications::attachment_search::AttachmentSearchPage>, ApiError>
{
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let store = AttachmentSearchStore::new(pool);
    Ok(Json(
        store
            .search(AttachmentSearchQuery {
                account_id: query.account_id.as_deref(),
                query: query.q.as_deref(),
                content_type: query.content_type.as_deref(),
                scan_status: query.scan_status.as_deref(),
                cursor: query.cursor.as_deref(),
                limit: query.limit.unwrap_or(100),
            })
            .await?,
    ))
}

#[derive(Serialize)]
pub(crate) struct AttachmentArchiveInspectionResponse {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) scan_status: String,
    pub(crate) report: ArchiveInspectionReport,
}

#[derive(Serialize)]
pub(crate) struct AttachmentPreviewResponse {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) scan_status: String,
    pub(crate) preview_kind: &'static str,
    pub(crate) text: String,
    pub(crate) data_url: Option<String>,
    pub(crate) truncated: bool,
    pub(crate) byte_count: usize,
    pub(crate) max_preview_bytes: usize,
}

pub(crate) async fn get_v1_attachment_preview(
    State(state): State<AppState>,
    Path(attachment_id): Path<String>,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    let attachment = mail_storage_store(&state)?
        .attachment_by_id(&attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if attachment.storage_kind != "local_fs" {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment preview requires a local blob",
        ));
    }
    if !is_preview_allowed_by_scan_status(&attachment) {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment preview is blocked by attachment scan status",
        ));
    }
    let preview_kind =
        attachment_preview_kind(&attachment).ok_or(ApiError::InvalidCommunicationQuery(
            "attachment preview supports text and image attachments only",
        ))?;

    let bytes = LocalMailBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
        .read_blob(&attachment.storage_path)
        .await?;
    let byte_count = bytes.len();

    match preview_kind {
        AttachmentPreviewKind::Text => text_attachment_preview(attachment, bytes, byte_count),
        AttachmentPreviewKind::Image => image_attachment_preview(attachment, bytes, byte_count),
    }
}

fn text_attachment_preview(
    attachment: StoredMailAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    let truncated = byte_count > MAX_TEXT_PREVIEW_BYTES;
    let preview_bytes = if truncated {
        &bytes[..MAX_TEXT_PREVIEW_BYTES]
    } else {
        &bytes
    };
    let text = String::from_utf8_lossy(preview_bytes).into_owned();

    Ok(Json(AttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "text",
        text,
        data_url: None,
        truncated,
        byte_count,
        max_preview_bytes: MAX_TEXT_PREVIEW_BYTES,
    }))
}

fn image_attachment_preview(
    attachment: StoredMailAttachmentWithBlob,
    bytes: Vec<u8>,
    byte_count: usize,
) -> Result<Json<AttachmentPreviewResponse>, ApiError> {
    if byte_count > MAX_IMAGE_PREVIEW_BYTES {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment image preview exceeds size limit",
        ));
    }
    let content_type = preview_image_content_type(&attachment).unwrap_or("image/png");
    let data_url = format!(
        "data:{content_type};base64,{}",
        BASE64_STANDARD.encode(bytes)
    );

    Ok(Json(AttachmentPreviewResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        preview_kind: "image",
        text: String::new(),
        data_url: Some(data_url),
        truncated: false,
        byte_count,
        max_preview_bytes: MAX_IMAGE_PREVIEW_BYTES,
    }))
}

pub(crate) async fn get_v1_attachment_archive_inspection(
    State(state): State<AppState>,
    Path(attachment_id): Path<String>,
) -> Result<Json<AttachmentArchiveInspectionResponse>, ApiError> {
    let attachment = mail_storage_store(&state)?
        .attachment_by_id(&attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if attachment.storage_kind != "local_fs" {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment archive inspection requires a local blob",
        ));
    }
    if !is_zip_attachment(&attachment) {
        return Err(ApiError::InvalidCommunicationQuery(
            "attachment archive inspection supports ZIP attachments only",
        ));
    }

    let bytes = LocalMailBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
        .read_blob(&attachment.storage_path)
        .await?;
    let report =
        inspect_zip_bytes(&bytes, ArchiveInspectionLimits::default()).map_err(|error| {
            tracing::warn!(
                attachment_id = %attachment.attachment.attachment_id,
                error = %error,
                "attachment archive inspection rejected archive"
            );
            ApiError::InvalidCommunicationQuery("attachment archive inspection failed")
        })?;

    Ok(Json(AttachmentArchiveInspectionResponse {
        attachment_id: attachment.attachment.attachment_id,
        message_id: attachment.attachment.message_id,
        filename: attachment.attachment.filename,
        content_type: attachment.attachment.content_type,
        scan_status: attachment.attachment.scan_status.as_str().to_owned(),
        report,
    }))
}

fn is_preview_allowed_by_scan_status(attachment: &StoredMailAttachmentWithBlob) -> bool {
    matches!(
        attachment.attachment.scan_status.as_str(),
        "not_scanned" | "clean"
    )
}

enum AttachmentPreviewKind {
    Text,
    Image,
}

fn attachment_preview_kind(
    attachment: &StoredMailAttachmentWithBlob,
) -> Option<AttachmentPreviewKind> {
    if is_previewable_text_attachment(attachment) {
        return Some(AttachmentPreviewKind::Text);
    }
    if is_previewable_image_attachment(attachment) {
        return Some(AttachmentPreviewKind::Image);
    }
    None
}

fn is_previewable_text_attachment(attachment: &StoredMailAttachmentWithBlob) -> bool {
    let content_type = attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase();
    if content_type.starts_with("text/") {
        return true;
    }
    matches!(
        content_type.as_str(),
        "application/json" | "application/xml" | "application/yaml" | "application/x-yaml"
    ) || attachment
        .attachment
        .filename
        .as_deref()
        .map(|filename| {
            let filename = filename.trim().to_ascii_lowercase();
            filename.ends_with(".txt")
                || filename.ends_with(".md")
                || filename.ends_with(".csv")
                || filename.ends_with(".json")
                || filename.ends_with(".xml")
                || filename.ends_with(".yaml")
                || filename.ends_with(".yml")
        })
        .unwrap_or(false)
}

fn is_previewable_image_attachment(attachment: &StoredMailAttachmentWithBlob) -> bool {
    preview_image_content_type(attachment).is_some()
}

fn preview_image_content_type(attachment: &StoredMailAttachmentWithBlob) -> Option<&'static str> {
    let content_type = attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase();
    match content_type.as_str() {
        "image/png" => Some("image/png"),
        "image/jpeg" => Some("image/jpeg"),
        "image/gif" => Some("image/gif"),
        "image/webp" => Some("image/webp"),
        _ => attachment
            .attachment
            .filename
            .as_deref()
            .and_then(|filename| {
                let filename = filename.trim().to_ascii_lowercase();
                if filename.ends_with(".png") {
                    Some("image/png")
                } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                    Some("image/jpeg")
                } else if filename.ends_with(".gif") {
                    Some("image/gif")
                } else if filename.ends_with(".webp") {
                    Some("image/webp")
                } else {
                    None
                }
            }),
    }
}

fn is_zip_attachment(attachment: &StoredMailAttachmentWithBlob) -> bool {
    let content_type = attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase();
    if content_type == "application/zip" || content_type == "application/x-zip-compressed" {
        return true;
    }
    attachment
        .attachment
        .filename
        .as_deref()
        .map(|filename| filename.to_ascii_lowercase().ends_with(".zip"))
        .unwrap_or(false)
}
