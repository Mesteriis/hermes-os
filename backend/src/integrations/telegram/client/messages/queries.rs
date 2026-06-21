use super::super::errors::TelegramError;
use super::super::models::TelegramMessage;
use super::super::rows::provider_channel_message_to_telegram_message;
use super::super::store::TelegramStore;
use super::super::validation::validate_message_list_limit;
use crate::platform::communications::ProviderChannelMessageStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

impl TelegramStore {
    pub async fn message_by_id(
        &self,
        message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .message_by_id(message_id, TELEGRAM_CHANNEL_KINDS)
            .await?
            .map(provider_channel_message_to_telegram_message))
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = validate_message_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .recent_messages(account_id, provider_chat_id, TELEGRAM_CHANNEL_KINDS, limit)
            .await?
            .into_iter()
            .map(provider_channel_message_to_telegram_message)
            .collect())
    }

    pub async fn messages_by_ids(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        if message_ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .messages_by_ids(message_ids, TELEGRAM_CHANNEL_KINDS)
            .await?
            .into_iter()
            .map(provider_channel_message_to_telegram_message)
            .collect())
    }
}
