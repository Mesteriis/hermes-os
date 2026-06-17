use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::ids::new_version_id;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::TelegramMessage;
use crate::integrations::telegram::client::models::messages::TelegramMessageVersion;
use crate::integrations::telegram::client::rows::row_to_telegram_message_version;

#[allow(clippy::too_many_arguments)]
pub async fn insert_message_version(
    pool: &PgPool,
    message_id: &str,
    account_id: &str,
    provider_message_id: &str,
    provider_chat_id: &str,
    version_number: i32,
    body_text: Option<&str>,
    edit_timestamp: DateTime<Utc>,
    source_event: Option<&str>,
    raw_diff: Value,
    provenance: Value,
) -> Result<TelegramMessageVersion, TelegramError> {
    let version_id = new_version_id();
    sqlx::query(
        r#"
        INSERT INTO telegram_message_versions
            (version_id, message_id, account_id, provider_message_id, provider_chat_id,
             version_number, body_text, edit_timestamp, source_event,
             raw_diff_payload, provenance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(&version_id)
    .bind(message_id)
    .bind(account_id)
    .bind(provider_message_id)
    .bind(provider_chat_id)
    .bind(version_number)
    .bind(body_text)
    .bind(edit_timestamp)
    .bind(source_event)
    .bind(&raw_diff)
    .bind(&provenance)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT * FROM telegram_message_versions WHERE version_id = $1")
        .bind(&version_id)
        .fetch_one(pool)
        .await?;

    row_to_telegram_message_version(row)
}

pub async fn list_message_versions(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageVersion>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_message_version)
        .collect()
}

pub async fn latest_message_version(
    pool: &PgPool,
    message_id: &str,
) -> Result<Option<TelegramMessageVersion>, TelegramError> {
    let row = sqlx::query(
        r#"
        SELECT *
        FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_telegram_message_version).transpose()
}

pub async fn latest_version_number(pool: &PgPool, message_id: &str) -> Result<i32, TelegramError> {
    let row: Option<(i32,)> = sqlx::query_as(
        r#"
        SELECT COALESCE(MAX(version_number), 0) as max_ver
        FROM telegram_message_versions
        WHERE message_id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(0))
}

pub async fn record_provider_edit_observation(
    pool: &PgPool,
    message: &TelegramMessage,
    body_text: &str,
    edit_timestamp: DateTime<Utc>,
    source_event: &str,
    raw_diff: Value,
    provenance: Value,
) -> Result<TelegramMessageVersion, TelegramError> {
    if let Some(existing) = latest_message_version(pool, &message.message_id).await?
        && existing.body_text.as_deref() == Some(body_text)
        && existing.source_event.as_deref() == Some(source_event)
        && existing.edit_timestamp == edit_timestamp
    {
        return Ok(existing);
    }

    let version_number = latest_version_number(pool, &message.message_id).await? + 1;
    insert_message_version(
        pool,
        &message.message_id,
        &message.account_id,
        &message.provider_message_id,
        message.provider_chat_id.as_deref().unwrap_or_default(),
        version_number,
        Some(body_text),
        edit_timestamp,
        Some(source_event),
        raw_diff,
        provenance,
    )
    .await
}

pub(crate) fn local_edit_diff(text: &str) -> Value {
    json!({"text_length": text.len()})
}
