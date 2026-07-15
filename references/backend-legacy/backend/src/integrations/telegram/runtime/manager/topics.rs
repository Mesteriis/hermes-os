use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::topics::NewTelegramTopic;

use super::super::commands::request_actor_get_forum_topics;
use super::super::status::account_runtime_kind;
use super::account::load_active_account;
use super::{TelegramRuntimeManager, TelegramRuntimeOperationContext};

pub(super) fn telegram_topic_id(telegram_chat_id: &str, provider_topic_id: i64) -> String {
    use sha2::{Digest, Sha256};
    let input = format!("{telegram_chat_id}\0{provider_topic_id}");
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("telegram_topic:v1:{:x}", hasher.finalize())
}

impl TelegramRuntimeManager {
    /// Fetches forum topics from TDLib for the given chat and upserts them into the projection.
    ///
    /// Returns the number of topics upserted. If the account has no active TDLib actor or runs
    /// in fixture mode, returns Ok(0) without error so the API can still serve DB rows.
    pub(crate) async fn sync_forum_topics<S>(
        &self,
        context: &TelegramRuntimeOperationContext<'_, S>,
        telegram_chat_id: &str,
    ) -> Result<usize, TelegramError>
    where
        S: crate::platform::secrets::resolver::SecretResolver + Sync + ?Sized,
    {
        let chat = context
            .telegram_store
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or(TelegramError::InvalidRequest(format!(
                "chat {telegram_chat_id} not found"
            )))?;

        let account = load_active_account(context.provider_account_store, &chat.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);

        if runtime_kind != "tdlib_qr_authorized" {
            return Ok(0);
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
                tracing::warn!(
                    error = %error,
                    telegram_chat_id,
                    "sync_forum_topics: failed to get TDLib actor, serving DB projection"
                );
                return Ok(0);
            }
        };

        let snapshots =
            request_actor_get_forum_topics(command_tx, chat.provider_chat_id.clone(), 100).await?;

        let mut upserted = 0;
        for snapshot in &snapshots {
            let new_topic = NewTelegramTopic {
                topic_id: telegram_topic_id(&chat.telegram_chat_id, snapshot.provider_topic_id),
                telegram_chat_id: chat.telegram_chat_id.clone(),
                account_id: chat.account_id.clone(),
                provider_topic_id: snapshot.provider_topic_id,
                provider_chat_id: chat.provider_chat_id.clone(),
                title: snapshot.title.clone(),
                icon_emoji: snapshot.icon_emoji.clone(),
                is_pinned: snapshot.is_pinned,
                is_closed: snapshot.is_closed,
                unread_count: topic_unread_count(snapshot.unread_count),
                last_message_at: snapshot.last_message_at,
            };
            crate::integrations::telegram::client::topics::upsert_topic(
                context.telegram_store.pool(),
                &new_topic,
            )
            .await?;
            upserted += 1;
        }

        Ok(upserted)
    }
}

fn topic_unread_count(unread_count: i64) -> i32 {
    unread_count.clamp(0, i32::MAX as i64) as i32
}
