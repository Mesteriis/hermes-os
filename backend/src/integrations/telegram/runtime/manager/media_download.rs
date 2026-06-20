use crate::integrations::telegram::client::TelegramError;
use crate::workflows::telegram_media_storage::{
    TelegramAttachmentAnchor, TelegramDownloadedFileData, TelegramMediaDownloadData,
    TelegramMediaDownloadProjection, TelegramMediaStorageError, persist_downloaded_media,
};

use super::super::commands::request_actor_download_file;
use super::super::models::{TelegramMediaDownloadRequest, TelegramMediaDownloadResponse};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramMediaDownloadContext, TelegramRuntimeManager, telegram_media_blob_root};
use crate::platform::secrets::SecretResolver;

impl TelegramRuntimeManager {
    pub(crate) async fn download_media<S: SecretResolver + Sync + ?Sized>(
        &self,
        context: TelegramMediaDownloadContext<'_, S>,
        request: &TelegramMediaDownloadRequest,
    ) -> Result<TelegramMediaDownloadResponse, TelegramError> {
        request.validate()?;
        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => Err(TelegramError::InvalidRequest(
                "Telegram media downloads require an enabled TDLib actor".to_owned(),
            )),
            "tdlib_qr_authorized" => {
                let command_tx = self
                    .ensure_tdlib_actor(
                        context.provider_secret_binding_store,
                        context.secret_store,
                        context.secret_resolver,
                        context.config,
                        &account,
                        context.event_bridge.clone(),
                    )
                    .await?;
                let file = request_actor_download_file(
                    command_tx,
                    request.tdlib_file_id,
                    request.priority.unwrap_or(16),
                )
                .await?;
                let anchor = if file.is_downloading_completed {
                    let anchor = context
                        .telegram_store
                        .attachment_anchor_for_message(
                            &request.account_id,
                            &request.provider_chat_id,
                            &request.provider_message_id,
                        )
                        .await?;
                    Some(TelegramAttachmentAnchor {
                        message_id: anchor.message_id,
                        raw_record_id: anchor.raw_record_id,
                    })
                } else {
                    None
                };
                let response = persist_downloaded_media(
                    context.telegram_store.pool().clone(),
                    &TelegramMediaDownloadData {
                        account_id: request.account_id.clone(),
                        provider_chat_id: request.provider_chat_id.clone(),
                        provider_message_id: request.provider_message_id.clone(),
                        tdlib_file_id: request.tdlib_file_id,
                        provider_attachment_id: request.provider_attachment_id.clone(),
                        filename: request.filename.clone(),
                        content_type: request.content_type.clone(),
                    },
                    &TelegramDownloadedFileData {
                        file_id: file.file_id,
                        size_bytes: file.size_bytes,
                        expected_size_bytes: file.expected_size_bytes,
                        local_path: file.local_path.clone(),
                        is_downloading_active: file.is_downloading_active,
                        is_downloading_completed: file.is_downloading_completed,
                        downloaded_size_bytes: file.downloaded_size_bytes,
                    },
                    anchor,
                    telegram_media_blob_root(),
                )
                .await
                .map_err(telegram_media_storage_error)?;
                let mut response = telegram_media_download_response(response);
                response.runtime_kind = runtime_kind;
                Ok(response)
            }
            "live_blocked" => Err(TelegramError::InvalidRequest(
                "account runtime is blocked until live TDLib is enabled".to_owned(),
            )),
            other => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram runtime `{other}`"
            ))),
        }
    }
}

fn telegram_media_download_response(
    projection: TelegramMediaDownloadProjection,
) -> TelegramMediaDownloadResponse {
    TelegramMediaDownloadResponse {
        account_id: projection.account_id,
        provider_chat_id: projection.provider_chat_id,
        provider_message_id: projection.provider_message_id,
        runtime_kind: projection.runtime_kind,
        status: projection.status,
        tdlib_file_id: projection.tdlib_file_id,
        local_path: projection.local_path,
        size_bytes: projection.size_bytes,
        expected_size_bytes: projection.expected_size_bytes,
        downloaded_size_bytes: projection.downloaded_size_bytes,
        is_downloading_active: projection.is_downloading_active,
        is_downloading_completed: projection.is_downloading_completed,
        attachment_id: projection.attachment_id,
        blob_id: projection.blob_id,
        scan_status: projection.scan_status,
    }
}

fn telegram_media_storage_error(error: TelegramMediaStorageError) -> TelegramError {
    match error {
        TelegramMediaStorageError::InvalidRequest(message) => {
            TelegramError::InvalidRequest(message)
        }
        TelegramMediaStorageError::Runtime(message) => TelegramError::TdlibRuntime(message),
        TelegramMediaStorageError::Storage(message) => TelegramError::MediaStorage(message),
    }
}
