use crate::integrations::telegram::client::{
    TelegramError, TelegramForwardRequest, TelegramManualSendRequest, TelegramManualSendResponse,
    TelegramReplyRequest, telegram_text_preview_hash,
};

use super::super::commands::{request_actor_forward, request_actor_reply, request_actor_send};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

impl TelegramRuntimeManager {
    pub(crate) async fn send_manual_message<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramManualSendRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account = load_active_account(context.communication_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => context.telegram_store.manual_send_message(request).await,
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
                let snapshot = request_actor_send(command_tx, request.clone()).await?;
                let import_batch_id = format!(
                    "telegram-manual-send:{}:{}",
                    account.account_id,
                    request.command_id.trim()
                );
                let result = context
                    .telegram_store
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

    pub(crate) async fn send_reply_message<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramReplyRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account = load_active_account(context.communication_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => Err(TelegramError::InvalidRequest(
                "reply command is not supported in fixture mode".to_owned(),
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
                let snapshot = request_actor_reply(
                    command_tx,
                    request.provider_chat_id.trim().to_owned(),
                    request.reply_to_provider_message_id.trim().to_owned(),
                    request.text.trim().to_owned(),
                    request.command_id.trim().to_owned(),
                )
                .await?;
                let import_batch_id = format!(
                    "telegram-reply:{}:{}",
                    account.account_id,
                    request.command_id.trim()
                );
                let result = context
                    .telegram_store
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

    pub(crate) async fn send_forward_message<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramForwardRequest,
    ) -> Result<TelegramManualSendResponse, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        request.validate()?;
        let account = load_active_account(context.communication_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        match runtime_kind.as_str() {
            "fixture" => Err(TelegramError::InvalidRequest(
                "forward command is not supported in fixture mode".to_owned(),
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
                let snapshot = request_actor_forward(
                    command_tx,
                    request.provider_chat_id.trim().to_owned(),
                    request.from_provider_chat_id.trim().to_owned(),
                    request.from_provider_message_id.trim().to_owned(),
                    request.command_id.trim().to_owned(),
                )
                .await?;
                let import_batch_id = format!(
                    "telegram-forward:{}:{}",
                    account.account_id,
                    request.command_id.trim()
                );
                let result = context
                    .telegram_store
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
                    rendered_preview_hash: telegram_text_preview_hash(&snapshot.text),
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
