use crate::integrations::telegram::tdjson::TelegramTdlibMessageSnapshot;

use super::super::errors::TelegramError;
use super::super::models::{NewTelegramMessage, TelegramChatKind, TelegramObservedMessage};
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(crate) async fn ingest_tdlib_message_snapshot(
        &self,
        account_id: &str,
        snapshot: &TelegramTdlibMessageSnapshot,
        import_batch_id: &str,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        let provider_account = self.telegram_provider_account(account_id).await?;
        let existing_chat = self
            .telegram_chat(&provider_account.account_id, &snapshot.provider_chat_id)
            .await?;
        let (chat_kind, chat_title) = match existing_chat {
            Some(chat) => (
                TelegramChatKind::try_from(chat.chat_kind.as_str())?,
                chat.title,
            ),
            None => (
                TelegramChatKind::Private,
                format!("Telegram Chat {}", snapshot.provider_chat_id),
            ),
        };
        let provider_message_id = format!(
            "{}:{}",
            snapshot.provider_chat_id, snapshot.provider_message_id
        );
        let message = NewTelegramMessage {
            account_id: provider_account.account_id,
            provider_chat_id: snapshot.provider_chat_id.clone(),
            provider_message_id,
            chat_kind,
            chat_title,
            sender_id: snapshot.sender_id.clone(),
            sender_display_name: snapshot.sender_display_name.clone(),
            text: snapshot.text.clone(),
            import_batch_id: import_batch_id.trim().to_owned(),
            occurred_at: snapshot.occurred_at,
            delivery_state: snapshot.delivery_state,
        };

        self.ingest_message_with_runtime(&message, "tdlib", Some(snapshot.raw.clone()))
            .await
    }
}
