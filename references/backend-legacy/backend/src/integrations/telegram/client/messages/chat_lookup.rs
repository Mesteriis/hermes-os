use super::super::errors::TelegramError;
use super::super::models::chats::TelegramChat;
use super::super::rows::row_to_telegram_chat;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(crate) async fn telegram_chat(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Result<Option<TelegramChat>, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE account_id = $1 AND provider_chat_id = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_chat_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_telegram_chat).transpose()
    }
}
