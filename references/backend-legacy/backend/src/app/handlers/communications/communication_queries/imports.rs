use super::super::*;
use crate::domains::communications::command_service::{
    CommunicationAttachmentImportCommand, CommunicationCommandService,
};
use crate::domains::communications::storage::models::ImportedCommunicationAttachment;

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
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let imported = CommunicationCommandService::new(pool)
        .import_attachment(CommunicationAttachmentImportCommand {
            account_id: request.account_id,
            channel_kind: request.channel_kind,
            filename: request.filename,
            content_type: request.content_type,
            content_base64: request.content_base64,
            source_kind: request.source_kind,
            metadata: request.metadata,
        })
        .await?;
    Ok(Json(import_response(imported)))
}

fn import_response(
    imported: ImportedCommunicationAttachment,
) -> CommunicationAttachmentImportResponse {
    CommunicationAttachmentImportResponse {
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
    }
}
