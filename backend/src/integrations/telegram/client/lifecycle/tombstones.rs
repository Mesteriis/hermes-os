use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::ids::new_tombstone_id;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::TelegramMessage;
use crate::integrations::telegram::client::models::messages::TelegramMessageTombstone;
use crate::integrations::telegram::client::rows::row_to_telegram_message_tombstone;

#[allow(clippy::too_many_arguments)]
pub async fn insert_tombstone(
    pool: &PgPool,
    message_id: &str,
    account_id: &str,
    provider_message_id: &str,
    provider_chat_id: &str,
    reason_class: &str,
    actor_class: &str,
    observed_at: DateTime<Utc>,
    source_event: Option<&str>,
    is_provider_delete: bool,
    is_local_visible: bool,
) -> Result<TelegramMessageTombstone, TelegramError> {
    let tombstone_id = new_tombstone_id();
    sqlx::query(
        r#"
        INSERT INTO telegram_message_tombstones
            (tombstone_id, message_id, account_id, provider_message_id, provider_chat_id,
             reason_class, actor_class, observed_at, source_event,
             is_provider_delete, is_local_visible)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(&tombstone_id)
    .bind(message_id)
    .bind(account_id)
    .bind(provider_message_id)
    .bind(provider_chat_id)
    .bind(reason_class)
    .bind(actor_class)
    .bind(observed_at)
    .bind(source_event)
    .bind(is_provider_delete)
    .bind(is_local_visible)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT * FROM telegram_message_tombstones WHERE tombstone_id = $1")
        .bind(&tombstone_id)
        .fetch_one(pool)
        .await?;

    row_to_telegram_message_tombstone(row)
}

pub async fn list_tombstones(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageTombstone>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_message_tombstone)
        .collect()
}

pub async fn is_message_visible(pool: &PgPool, message_id: &str) -> Result<bool, TelegramError> {
    let row: Option<(bool,)> = sqlx::query_as(
        r#"
        SELECT is_local_visible
        FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(true))
}

pub async fn record_provider_delete_observation(
    pool: &PgPool,
    message: &TelegramMessage,
    observed_at: DateTime<Utc>,
    source_event: &str,
    is_provider_delete: bool,
    from_cache: bool,
) -> Result<TelegramMessageTombstone, TelegramError> {
    let latest = sqlx::query(
        r#"
        SELECT *
        FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&message.message_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = latest {
        let tombstone = row_to_telegram_message_tombstone(row)?;
        if tombstone.reason_class == "deleted_by_provider"
            && tombstone.actor_class == "provider"
            && !tombstone.is_local_visible
        {
            return Ok(tombstone);
        }
    }

    let tombstone_id = new_tombstone_id();
    sqlx::query(
        r#"
        INSERT INTO telegram_message_tombstones
            (tombstone_id, message_id, account_id, provider_message_id, provider_chat_id,
             reason_class, actor_class, observed_at, source_event,
             is_provider_delete, is_local_visible, metadata, provenance)
        VALUES ($1, $2, $3, $4, $5, 'deleted_by_provider', 'provider', $6, $7, $8, false, $9, $10)
        "#,
    )
    .bind(&tombstone_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_message_id)
    .bind(message.provider_chat_id.as_deref().unwrap_or_default())
    .bind(observed_at)
    .bind(source_event)
    .bind(is_provider_delete)
    .bind(json!({
        "from_cache": from_cache,
        "provider_delete": is_provider_delete,
    }))
    .bind(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "source": source_event,
    }))
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT * FROM telegram_message_tombstones WHERE tombstone_id = $1")
        .bind(&tombstone_id)
        .fetch_one(pool)
        .await?;

    row_to_telegram_message_tombstone(row)
}
