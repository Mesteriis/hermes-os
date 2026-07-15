use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::command_conversion::row_to_whatsapp_provider_write_command;
use super::command_execution::mirror_canonical_provider_command_for_pool;
use super::retry::next_attempt_at;
use super::{WhatsAppProviderWriteCommand, WhatsappWebError};

pub(crate) async fn reschedule_failed_command(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    error_message: &str,
    error_code: Option<&str>,
    retry_after_seconds: Option<i64>,
) -> Result<Option<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let current_retry_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT retry_count
        FROM whatsapp_provider_write_commands
        WHERE command_id = $1 AND status = 'executing'
        "#,
    )
    .bind(command_id)
    .fetch_optional(pool)
    .await?;
    let Some(retry_count) = current_retry_count else {
        return Ok(None);
    };
    let next_attempt_at = next_attempt_at(now, retry_count, retry_after_seconds);
    let failure_result_payload = json!({
        "failure": {
            "error_message": error_message,
            "error_code": error_code,
            "retry_after_seconds": retry_after_seconds,
            "reported_at": now,
            "reported_via": "runtime_bridge_failed",
        }
    });
    let failure_provider_state = json!({
        "last_failure": {
            "error_message": error_message,
            "error_code": error_code,
            "retry_after_seconds": retry_after_seconds,
            "reported_at": now,
            "reported_via": "runtime_bridge_failed",
        }
    });
    let row = sqlx::query(
        r#"
        UPDATE whatsapp_provider_write_commands
        SET status = CASE WHEN retry_count >= max_retries THEN 'dead_letter' ELSE 'retrying' END,
            next_attempt_at = CASE WHEN retry_count >= max_retries THEN next_attempt_at ELSE $3 END,
            locked_at = NULL, locked_by = NULL, last_error = $4,
            result_payload = COALESCE(result_payload, '{}'::jsonb) || $5::jsonb,
            provider_state = COALESCE(provider_state, '{}'::jsonb) || $6::jsonb,
            reconciliation_status = 'not_observed',
            dead_lettered_at = CASE WHEN retry_count >= max_retries THEN $2 ELSE dead_lettered_at END,
            updated_at = $2
        WHERE command_id = $1 AND status = 'executing'
        RETURNING *
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(next_attempt_at)
    .bind(error_message)
    .bind(failure_result_payload)
    .bind(failure_provider_state)
    .fetch_optional(pool)
    .await?;
    let command = row
        .map(row_to_whatsapp_provider_write_command)
        .transpose()?;
    if let Some(command) = &command {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(command)
}

pub(crate) async fn dead_letter_failed_command(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    error_message: &str,
    error_code: Option<&str>,
) -> Result<Option<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let failure_result_payload = json!({
        "failure": {
            "error_message": error_message,
            "error_code": error_code,
            "retry_after_seconds": null,
            "reported_at": now,
            "reported_via": "runtime_bridge_terminal_failed",
            "retry_policy": "terminal",
        }
    });
    let failure_provider_state = json!({
        "last_failure": {
            "error_message": error_message,
            "error_code": error_code,
            "retry_after_seconds": null,
            "reported_at": now,
            "reported_via": "runtime_bridge_terminal_failed",
            "retry_policy": "terminal",
        }
    });
    let row = sqlx::query(
        r#"
        UPDATE whatsapp_provider_write_commands
        SET status = 'dead_letter', next_attempt_at = NULL, locked_at = NULL, locked_by = NULL,
            last_error = $3,
            result_payload = COALESCE(result_payload, '{}'::jsonb) || $4::jsonb,
            provider_state = COALESCE(provider_state, '{}'::jsonb) || $5::jsonb,
            reconciliation_status = 'not_observed', dead_lettered_at = $2, updated_at = $2
        WHERE command_id = $1 AND status = 'executing'
        RETURNING *
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(error_message)
    .bind(failure_result_payload)
    .bind(failure_provider_state)
    .fetch_optional(pool)
    .await?;
    let command = row
        .map(row_to_whatsapp_provider_write_command)
        .transpose()?;
    if let Some(command) = &command {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(command)
}

pub(crate) async fn record_live_provider_command_submitted(
    pool: &PgPool,
    now: DateTime<Utc>,
    outcome: &super::WhatsAppProviderCommandExecutionOutcome,
) -> Result<Option<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let row = sqlx::query(
        r#"
        UPDATE whatsapp_provider_write_commands
        SET status = 'executing', locked_at = NULL, locked_by = NULL, last_error = NULL,
            result_payload = COALESCE(result_payload, '{}'::jsonb) || $3::jsonb,
            provider_state = COALESCE(provider_state, '{}'::jsonb) || $4::jsonb,
            reconciliation_status = 'awaiting_provider', updated_at = $2
        WHERE command_id = $1 AND status = 'executing'
        RETURNING *
        "#,
    )
    .bind(&outcome.command_id)
    .bind(now)
    .bind(&outcome.result_payload)
    .bind(&outcome.provider_state)
    .fetch_optional(pool)
    .await?;
    let command = row
        .map(row_to_whatsapp_provider_write_command)
        .transpose()?;
    if let Some(command) = &command {
        mirror_canonical_provider_command_for_pool(pool, command).await?;
    }
    Ok(command)
}

pub(super) async fn recover_stale_executing_commands_scoped(
    pool: &PgPool,
    now: DateTime<Utc>,
    account_ids: &[String],
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    if account_ids.is_empty() {
        return Ok(Vec::new());
    }
    let stale_before = now - chrono::Duration::seconds(super::STALE_EXECUTION_LOCK_SECONDS);
    let stale_rows = sqlx::query(
        "SELECT command.* FROM whatsapp_provider_write_commands command WHERE status = 'executing' AND command.locked_at IS NOT NULL AND command.locked_at <= $1 AND command.account_id = ANY($2) ORDER BY command.locked_at ASC, command.command_id ASC",
    )
    .bind(stale_before)
    .bind(account_ids)
    .fetch_all(pool)
    .await?;
    let stale_commands = stale_rows
        .into_iter()
        .map(row_to_whatsapp_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    let mut recovered = Vec::with_capacity(stale_commands.len());
    for command in stale_commands {
        let retry_after_seconds = (command.retry_count < command.max_retries)
            .then(|| super::retry::retry_delay_seconds(command.retry_count, None));
        let failure_result_payload = json!({
            "failure": { "error_message": "WhatsApp provider command execution was interrupted before provider reconciliation", "error_code": "interrupted_execution", "retry_after_seconds": retry_after_seconds, "reported_at": now, "reported_via": "stale_execution_recovery" }
        });
        let failure_provider_state = json!({
            "last_failure": { "error_message": "WhatsApp provider command execution was interrupted before provider reconciliation", "error_code": "interrupted_execution", "retry_after_seconds": retry_after_seconds, "reported_at": now, "reported_via": "stale_execution_recovery" }
        });
        let row = sqlx::query(
            "UPDATE whatsapp_provider_write_commands SET status = CASE WHEN retry_count >= max_retries THEN 'dead_letter' ELSE 'retrying' END, next_attempt_at = CASE WHEN retry_count >= max_retries THEN next_attempt_at ELSE $2 END, locked_at = NULL, locked_by = NULL, last_error = 'WhatsApp provider command execution was interrupted before provider reconciliation', result_payload = COALESCE(result_payload, '{}'::jsonb) || $3::jsonb, provider_state = COALESCE(provider_state, '{}'::jsonb) || $4::jsonb, reconciliation_status = 'not_observed', dead_lettered_at = CASE WHEN retry_count >= max_retries THEN $5 ELSE dead_lettered_at END, updated_at = $5 WHERE command_id = $1 AND status = 'executing' RETURNING *",
        )
        .bind(&command.command_id)
        .bind(next_attempt_at(now, command.retry_count, None))
        .bind(failure_result_payload)
        .bind(failure_provider_state)
        .bind(now)
        .fetch_optional(pool)
        .await?;
        if let Some(row) = row {
            let updated = row_to_whatsapp_provider_write_command(row)?;
            mirror_canonical_provider_command_for_pool(pool, &updated).await?;
            recovered.push(updated);
        }
    }
    Ok(recovered)
}
