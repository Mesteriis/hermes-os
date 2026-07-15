use crate::integrations::telegram::client::errors::TelegramError;

use super::super::commands::request_actor_download_file;
use super::super::models::{TelegramMediaDownloadRequest, TelegramMediaDownloadResponse};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramMediaDownloadContext, TelegramRuntimeManager};
use crate::platform::secrets::resolver::SecretResolver;

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
                Ok(TelegramMediaDownloadResponse {
                    account_id: request.account_id.clone(),
                    provider_chat_id: request.provider_chat_id.clone(),
                    provider_message_id: request.provider_message_id.clone(),
                    runtime_kind,
                    status: if file.is_downloading_completed {
                        "downloaded".to_owned()
                    } else if file.is_downloading_active {
                        "downloading".to_owned()
                    } else {
                        "remote".to_owned()
                    },
                    tdlib_file_id: file.file_id,
                    local_path: file.local_path,
                    size_bytes: file.size_bytes,
                    expected_size_bytes: file.expected_size_bytes,
                    downloaded_size_bytes: file.downloaded_size_bytes,
                    is_downloading_active: file.is_downloading_active,
                    is_downloading_completed: file.is_downloading_completed,
                    attachment_id: None,
                    blob_id: None,
                    scan_status: None,
                })
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
