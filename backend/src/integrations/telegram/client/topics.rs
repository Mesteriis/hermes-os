use chrono::Utc;
use sqlx::PgPool;

use super::errors::TelegramError;
use super::models::topics::{NewTelegramTopic, TelegramTopic};

fn row_to_telegram_topic(row: sqlx::postgres::PgRow) -> Result<TelegramTopic, TelegramError> {
    use sqlx::Row;
    Ok(TelegramTopic {
        topic_id: row.try_get("topic_id")?,
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        account_id: row.try_get("account_id")?,
        provider_topic_id: row.try_get("provider_topic_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        title: row.try_get("title")?,
        icon_emoji: row.try_get("icon_emoji")?,
        is_pinned: row.try_get("is_pinned")?,
        is_closed: row.try_get("is_closed")?,
        unread_count: row.try_get("unread_count")?,
        last_message_at: row.try_get("last_message_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn upsert_topic(
    pool: &PgPool,
    topic: &NewTelegramTopic,
) -> Result<TelegramTopic, TelegramError> {
    let now = Utc::now();
    let row = sqlx::query(
        r"
        INSERT INTO telegram_topics (
            topic_id, telegram_chat_id, account_id, provider_topic_id, provider_chat_id,
            title, icon_emoji, is_pinned, is_closed, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
        ON CONFLICT (telegram_chat_id, provider_topic_id)
        DO UPDATE SET
            title        = EXCLUDED.title,
            icon_emoji   = EXCLUDED.icon_emoji,
            is_pinned    = EXCLUDED.is_pinned,
            is_closed    = EXCLUDED.is_closed,
            updated_at   = EXCLUDED.updated_at
        RETURNING *
        ",
    )
    .bind(&topic.topic_id)
    .bind(&topic.telegram_chat_id)
    .bind(&topic.account_id)
    .bind(topic.provider_topic_id)
    .bind(&topic.provider_chat_id)
    .bind(&topic.title)
    .bind(&topic.icon_emoji)
    .bind(topic.is_pinned)
    .bind(topic.is_closed)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(TelegramError::from)?;

    row_to_telegram_topic(row)
}

pub async fn list_topics(
    pool: &PgPool,
    telegram_chat_id: &str,
    limit: i64,
) -> Result<Vec<TelegramTopic>, TelegramError> {
    let rows = sqlx::query(
        r"
        SELECT * FROM telegram_topics
        WHERE telegram_chat_id = $1
        ORDER BY is_pinned DESC, last_message_at DESC NULLS LAST, updated_at DESC
        LIMIT $2
        ",
    )
    .bind(telegram_chat_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    rows.into_iter().map(row_to_telegram_topic).collect()
}

pub async fn get_topic(
    pool: &PgPool,
    topic_id: &str,
) -> Result<Option<TelegramTopic>, TelegramError> {
    let row = sqlx::query("SELECT * FROM telegram_topics WHERE topic_id = $1")
        .bind(topic_id)
        .fetch_optional(pool)
        .await
        .map_err(TelegramError::from)?;

    row.map(row_to_telegram_topic).transpose()
}

pub async fn list_topic_message_ids(
    pool: &PgPool,
    topic_id: &str,
    limit: i64,
) -> Result<Vec<String>, TelegramError> {
    // Messages belong to a forum topic via message_metadata->>'forum_topic_id'.
    // The index idx_comm_messages_forum_topic_id covers this filter.
    let rows: Vec<(String,)> = sqlx::query_as(
        r"
        SELECT message_id FROM communication_messages
        WHERE message_metadata->>'forum_topic_id' = $1
          AND channel_kind IN ('telegram_user', 'telegram_bot')
        ORDER BY COALESCE(occurred_at, projected_at) DESC NULLS LAST, message_id ASC
        LIMIT $2
        ",
    )
    .bind(topic_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}
