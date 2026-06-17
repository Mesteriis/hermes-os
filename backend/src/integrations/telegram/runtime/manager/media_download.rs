use crate::integrations::telegram::client::TelegramError;

use super::super::commands::request_actor_download_file;
use super::super::media::persist_downloaded_media;
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
        let account = load_active_account(context.communication_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => Err(TelegramError::InvalidRequest(
                "Telegram media downloads require an enabled TDLib actor".to_owned(),
            )),
            "tdlib_qr_authorized" => {
                let command_tx = self
                    .ensure_tdlib_actor(
                        context.communication_store,
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
                let mut response = persist_downloaded_media(
                    context.telegram_store,
                    context.mail_store,
                    request,
                    &file,
                    telegram_media_blob_root(),
                )
                .await?;
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
