use sqlx::PgPool;

use super::errors::TelegramError;
use super::models::TelegramMessage;
use super::rows::provider_channel_message_to_telegram_message;
use crate::platform::communications::ProviderChannelMessageStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

pub async fn search_messages(
    pool: &PgPool,
    account_id: Option<&str>,
    provider_chat_id: Option<&str>,
    query: &str,
    limit: i64,
) -> Result<Vec<TelegramMessage>, TelegramError> {
    Ok(ProviderChannelMessageStore::new(pool.clone())
        .search_messages(account_id, provider_chat_id, query, TELEGRAM_CHANNEL_KINDS, limit)
        .await?
        .into_iter()
        .map(provider_channel_message_to_telegram_message)
        .collect())
}

pub async fn search_chats(
    pool: &PgPool,
    account_id: Option<&str>,
    query: &str,
    limit: i64,
) -> Result<Vec<super::models::TelegramChat>, TelegramError> {
    let like_pattern = format!("%{}%", query);
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
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(super::rows::row_to_telegram_chat).collect()
}
