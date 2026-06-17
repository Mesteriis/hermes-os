use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::rows::row_to_telegram_provider_write_command;

const EDIT_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different message body than requested";
const PIN_PROVIDER_MISMATCH_ERROR: &str = "Provider observed a different pin state than requested";

pub async fn reconcile_edit_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    body_text: &str,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND provider_message_id = $3
          AND command_kind = 'edit'
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let Some(new_text) = command
            .payload
            .get("new_text")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
        else {
            continue;
        };
        if new_text != body_text {
            let provider_state = json!({
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "expected_body_text": new_text,
                "observed_body_text": body_text,
                "observed_via": observed_via,
            });
            let result_payload = json!({
                "source": observed_via,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "expected_body_text": new_text,
                "observed_body_text": body_text,
                "provider_observed_at": observed_at,
                "mismatch": true,
            });
            let row = sqlx::query(
                r#"
                UPDATE telegram_provider_write_commands
                SET status = 'failed',
                    result_payload = $3,
                    last_error = $4,
                    provider_observed_at = $2,
                    provider_state = $5,
                    reconciliation_status = 'mismatch',
                    reconciled_at = $2,
                    completed_at = NULL,
                    locked_at = NULL,
                    locked_by = NULL,
                    next_attempt_at = NULL,
                    dead_lettered_at = NULL,
                    updated_at = $2
                WHERE command_id = $1
                RETURNING *
                "#,
            )
            .bind(&command.command_id)
            .bind(observed_at)
            .bind(&result_payload)
            .bind(EDIT_PROVIDER_MISMATCH_ERROR)
            .bind(&provider_state)
            .fetch_one(pool)
            .await?;
            reconciled.push(row_to_telegram_provider_write_command(row)?);
            continue;
        }

        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "body_text": body_text,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "body_text": body_text,
            "provider_observed_at": observed_at,
        });
        let row = sqlx::query(
            r#"
            UPDATE telegram_provider_write_commands
            SET status = 'completed',
                result_payload = $3,
                last_error = NULL,
                provider_observed_at = $2,
                provider_state = $4,
                reconciliation_status = 'observed',
                reconciled_at = $2,
                completed_at = $2,
                locked_at = NULL,
                locked_by = NULL,
                next_attempt_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $2
            WHERE command_id = $1
            RETURNING *
            "#,
        )
        .bind(&command.command_id)
        .bind(observed_at)
        .bind(&result_payload)
        .bind(&provider_state)
        .fetch_one(pool)
        .await?;
        reconciled.push(row_to_telegram_provider_write_command(row)?);
    }

    Ok(reconciled)
}

pub async fn reconcile_message_pin_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    is_pinned: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND provider_message_id = $3
          AND command_kind IN ('pin', 'unpin')
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let expected_is_pinned = match command.command_kind.as_str() {
            "pin" => Some(true),
            "unpin" => Some(false),
            _ => None,
        };
        let Some(expected_is_pinned) = expected_is_pinned else {
            continue;
        };
        if expected_is_pinned != is_pinned {
            let provider_state = json!({
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "expected_is_pinned": expected_is_pinned,
                "observed_is_pinned": is_pinned,
                "observed_via": observed_via,
            });
            let result_payload = json!({
                "source": observed_via,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "expected_is_pinned": expected_is_pinned,
                "observed_is_pinned": is_pinned,
                "provider_observed_at": observed_at,
                "mismatch": true,
            });
            let row = sqlx::query(
                r#"
                UPDATE telegram_provider_write_commands
                SET status = 'failed',
                    result_payload = $3,
                    last_error = $4,
                    provider_observed_at = $2,
                    provider_state = $5,
                    reconciliation_status = 'mismatch',
                    reconciled_at = $2,
                    completed_at = NULL,
                    locked_at = NULL,
                    locked_by = NULL,
                    next_attempt_at = NULL,
                    dead_lettered_at = NULL,
                    updated_at = $2
                WHERE command_id = $1
                RETURNING *
                "#,
            )
            .bind(&command.command_id)
            .bind(observed_at)
            .bind(&result_payload)
            .bind(PIN_PROVIDER_MISMATCH_ERROR)
            .bind(&provider_state)
            .fetch_one(pool)
            .await?;
            reconciled.push(row_to_telegram_provider_write_command(row)?);
            continue;
        }
        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "is_pinned": is_pinned,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "is_pinned": is_pinned,
            "provider_observed_at": observed_at,
        });
        let row = sqlx::query(
            r#"
            UPDATE telegram_provider_write_commands
            SET status = 'completed',
                result_payload = $3,
                last_error = NULL,
                provider_observed_at = $2,
                provider_state = $4,
                reconciliation_status = 'observed',
                reconciled_at = $2,
                completed_at = $2,
                locked_at = NULL,
                locked_by = NULL,
                next_attempt_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $2
            WHERE command_id = $1
            RETURNING *
            "#,
        )
        .bind(&command.command_id)
        .bind(observed_at)
        .bind(&result_payload)
        .bind(&provider_state)
        .fetch_one(pool)
        .await?;
        reconciled.push(row_to_telegram_provider_write_command(row)?);
    }

    Ok(reconciled)
}

pub async fn reconcile_delete_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND provider_message_id = $3
          AND command_kind = 'delete'
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "is_deleted": true,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "is_deleted": true,
            "provider_observed_at": observed_at,
        });
        let row = sqlx::query(
            r#"
            UPDATE telegram_provider_write_commands
            SET status = 'completed',
                result_payload = $3,
                last_error = NULL,
                provider_observed_at = $2,
                provider_state = $4,
                reconciliation_status = 'observed',
                reconciled_at = $2,
                completed_at = $2,
                locked_at = NULL,
                locked_by = NULL,
                next_attempt_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $2
            WHERE command_id = $1
            RETURNING *
            "#,
        )
        .bind(&command.command_id)
        .bind(observed_at)
        .bind(&result_payload)
        .bind(&provider_state)
        .fetch_one(pool)
        .await?;
        reconciled.push(row_to_telegram_provider_write_command(row)?);
    }

    Ok(reconciled)
}
