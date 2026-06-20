use sqlx::PgPool;

use super::errors::TelegramError;
use super::models::{TelegramChat, TelegramMessage};
use super::rows::{provider_channel_message_to_telegram_message, row_to_telegram_chat};
use super::store::TelegramStore;
use crate::platform::communications::ProviderChannelMessageStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

impl TelegramStore {
    pub async fn pinned_messages(
        &self,
        telegram_chat_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = super::validation::validate_message_list_limit(limit)?;
        let chat = self
            .telegram_chat_by_id(telegram_chat_id.trim())
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?;

        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .pinned_messages(
                &chat.account_id,
                &chat.provider_chat_id,
                TELEGRAM_CHANNEL_KINDS,
                limit,
            )
            .await?
            .into_iter()
            .map(provider_channel_message_to_telegram_message)
            .collect())
    }

    pub async fn search_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = super::validation::validate_message_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|v| !v.is_empty());
        let provider_chat_id = provider_chat_id.map(str::trim).filter(|v| !v.is_empty());

        Ok(ProviderChannelMessageStore::new(self.pool.clone())
            .search_messages(
                account_id,
                provider_chat_id,
                query,
                TELEGRAM_CHANNEL_KINDS,
                limit,
            )
            .await?
            .into_iter()
            .map(provider_channel_message_to_telegram_message)
            .collect())
    }

    pub async fn search_chats(
        &self,
        account_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let like_pattern = format!("%{query}%");
        let account_id = account_id.map(str::trim).filter(|v| !v.is_empty());

        let rows = sqlx::query(
            r#"
            SELECT * FROM telegram_chats
            WHERE title ILIKE $1
              AND ($2::text IS NULL OR account_id = $2)
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
        )
        .bind(&like_pattern)
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_chat).collect()
    }
}
