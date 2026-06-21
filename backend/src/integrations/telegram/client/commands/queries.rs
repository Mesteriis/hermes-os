use sqlx::PgPool;

use super::super::errors::TelegramError;
use super::super::models::messages::TelegramProviderWriteCommand;
use super::super::rows::row_to_telegram_provider_write_command;

pub async fn find_command_by_idempotency(
    pool: &PgPool,
    account_id: &str,
    idempotency_key: &str,
) -> Result<Option<TelegramProviderWriteCommand>, TelegramError> {
    let row = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(account_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_telegram_provider_write_command).transpose()
}

pub async fn list_commands(
    pool: &PgPool,
    account_id: &str,
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    list_commands_filtered(pool, account_id, None, None, &[], limit).await
}

pub async fn list_commands_filtered(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: Option<&str>,
    provider_message_id: Option<&str>,
    command_kinds: &[String],
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND ($2::text IS NULL OR provider_chat_id = $2)
          AND ($3::text IS NULL OR provider_message_id = $3)
          AND (cardinality($4::text[]) = 0 OR command_kind = ANY($4::text[]))
        ORDER BY created_at DESC
        LIMIT $5
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .bind(command_kinds)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect()
}

pub async fn list_queued_commands_for_execution(
    pool: &PgPool,
    account_id: &str,
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND status IN ('queued', 'retrying')
          AND retry_count < max_retries
          AND (next_attempt_at IS NULL OR next_attempt_at <= now())
          AND command_kind IN (
              'send_text', 'send_media', 'reply', 'forward',
              'edit', 'delete', 'react', 'unreact', 'pin', 'unpin',
              'mark_read', 'mark_unread', 'archive', 'unarchive',
              'mute', 'unmute', 'join', 'leave', 'folder_add', 'folder_remove',
              'admin_action'
          )
        ORDER BY COALESCE(next_attempt_at, created_at) ASC, created_at ASC, command_id ASC
        LIMIT $2
        "#,
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect()
}
