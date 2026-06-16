use sqlx::PgPool;

use super::errors::TelegramError;
use super::models::{TelegramChat, TelegramMessage};
use super::rows::{row_to_telegram_chat, row_to_telegram_message};
use super::store::TelegramStore;

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

        let rows = sqlx::query(
            r#"
            SELECT
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, body_text, occurred_at, projected_at,
                channel_kind, conversation_id, sender_display_name,
                delivery_state, message_metadata
            FROM communication_messages
            WHERE channel_kind IN ('telegram_user', 'telegram_bot')
              AND account_id = $1
              AND conversation_id = $2
              AND (
                COALESCE(message_metadata->>'is_pinned', 'false') = 'true'
                OR COALESCE(message_metadata->>'pinned', 'false') = 'true'
              )
            ORDER BY COALESCE(occurred_at, projected_at) DESC
            LIMIT $3
            "#,
        )
        .bind(&chat.account_id)
        .bind(&chat.provider_chat_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_message).collect()
    }

    pub async fn search_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = super::validation::validate_message_list_limit(limit)?;
        let like_pattern = format!("%{}%", query);
        let account_id = account_id.map(str::trim).filter(|v| !v.is_empty());
        let provider_chat_id = provider_chat_id.map(str::trim).filter(|v| !v.is_empty());

        let rows = sqlx::query(
            r#"
            SELECT
                message_id, raw_record_id, account_id, provider_record_id,
                subject, sender, body_text, occurred_at, projected_at,
                channel_kind, conversation_id, sender_display_name,
                delivery_state, message_metadata
            FROM communication_messages
            WHERE channel_kind IN ('telegram_user', 'telegram_bot')
              AND body_text ILIKE $1
              AND ($2::text IS NULL OR account_id = $2)
              AND ($3::text IS NULL OR conversation_id = $3)
            ORDER BY COALESCE(occurred_at, projected_at) DESC
            LIMIT $4
            "#,
        )
        .bind(&like_pattern)
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_message).collect()
    }

    pub async fn search_chats(
        &self,
        account_id: Option<&str>,
        query: &str,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let like_pattern = format!("%{}%", query);
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
