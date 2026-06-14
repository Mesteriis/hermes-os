use crate::domains::mail::core::CommunicationIngestionStore;
use crate::integrations::telegram::client::{
    TelegramError, TelegramManualSendRequest, TelegramManualSendResponse, TelegramStore,
    telegram_text_preview_hash,
};
use crate::platform::config::AppConfig;
use crate::platform::secrets::{SecretReferenceStore, SecretResolver};

use super::super::commands::request_actor_send;
use super::super::status::account_runtime_kind;
use super::TelegramRuntimeManager;
use super::account::load_active_account;

impl TelegramRuntimeManager {
    pub async fn send_manual_message(
        &self,
        communication_store: &CommunicationIngestionStore,
        telegram_store: &TelegramStore,
        secret_store: &SecretReferenceStore,
        secret_resolver: &(impl SecretResolver + Sync + ?Sized),
        config: &AppConfig,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError> {
        request.validate()?;
        let account = load_active_account(communication_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => telegram_store.manual_send_message(request).await,
            "tdlib_qr_authorized" => {
                let command_tx = self
                    .ensure_tdlib_actor(
                        communication_store,
                        secret_store,
                        secret_resolver,
                        config,
                        &account,
                    )
                    .await?;
                let snapshot = request_actor_send(command_tx, request.clone()).await?;
                let import_batch_id = format!(
                    "telegram-manual-send:{}:{}",
                    account.account_id,
                    request.command_id.trim()
                );
                let result = telegram_store
                    .ingest_tdlib_message_snapshot(&account.account_id, &snapshot, &import_batch_id)
                    .await?;
                Ok(TelegramManualSendResponse {
                    raw_record_id: result.raw_record_id,
                    message_id: result.message_id,
                    account_id: account.account_id,
                    provider_chat_id: request.provider_chat_id.trim().to_owned(),
                    delivery_state: snapshot.delivery_state.as_str().to_owned(),
                    status: "sent".to_owned(),
                    runtime_kind,
                    rendered_preview_hash: telegram_text_preview_hash(&request.text),
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
