use crate::integrations::telegram::client::TelegramError;

use super::super::commands::{request_actor_search_chat_messages, request_actor_search_messages};
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

pub struct TelegramProviderSearchRequest {
    pub account_id: String,
    pub provider_chat_id: Option<String>,
    pub query: String,
    pub limit: i32,
}

impl TelegramRuntimeManager {
    /// Calls TDLib `searchMessages` or `searchChatMessages` and ingests results.
    ///
    /// Returns ingested message IDs. Falls back to Ok(vec![]) for fixture mode or when no
    /// active actor is available.
    pub(crate) async fn search_provider_messages<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        request: &TelegramProviderSearchRequest,
    ) -> Result<Vec<String>, TelegramError>
    where
        S: crate::platform::secrets::SecretResolver + Sync + ?Sized,
    {
        if request.query.trim().is_empty() {
            return Ok(vec![]);
        }

        let account =
            load_active_account(context.provider_account_store, &request.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);

        if runtime_kind != "tdlib_qr_authorized" {
            return Ok(vec![]);
        }

        let command_tx = match self
            .ensure_tdlib_actor(
                context.provider_secret_binding_store,
                context.secret_store,
                context.secret_resolver,
                context.config,
                &account,
                context.event_bridge.clone(),
            )
            .await
        {
            Ok(tx) => tx,
            Err(error) => {
                tracing::debug!(
                    error = %error,
                    account_id = %request.account_id,
                    "search_provider_messages: TDLib actor unavailable"
                );
                return Ok(vec![]);
            }
        };

        let snapshots = if let Some(provider_chat_id) = &request.provider_chat_id {
            request_actor_search_chat_messages(
                command_tx,
                provider_chat_id.clone(),
                request.query.clone(),
                request.limit,
            )
            .await?
        } else {
            request_actor_search_messages(command_tx, request.query.clone(), request.limit).await?
        };

        let import_batch_id = format!(
            "telegram-search:{}:{}",
            request.account_id,
            &request.query[..request.query.len().min(32)]
        );

        let mut message_ids = Vec::with_capacity(snapshots.len());
        for snapshot in &snapshots {
            match context
                .telegram_store
                .ingest_tdlib_message_snapshot(&request.account_id, snapshot, &import_batch_id)
                .await
            {
                Ok(result) => {
                    if let Err(error) = context
                        .telegram_store
                        .publish_observed_message_raw_signal(
                            &result,
                            context
                                .event_bridge
                                .as_ref()
                                .map(|bridge| &bridge.event_bus),
                        )
                        .await
                    {
                        tracing::warn!(
                            error = %error,
                            provider_message_id = %snapshot.provider_message_id,
                            "search_provider_messages: failed to publish Signal Hub raw signal"
                        );
                    }
                    message_ids.push(result.message_id);
                }
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        provider_message_id = %snapshot.provider_message_id,
                        "search_provider_messages: failed to ingest snapshot"
                    );
                }
            }
        }

        Ok(message_ids)
    }
}
