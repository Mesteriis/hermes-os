use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::commands::{mark_command_mismatch, mark_command_reconciled};
use super::errors::TelegramError;
use super::models::messages::TelegramProviderWriteCommand;
use super::rows::row_to_telegram_provider_write_command;

const PROVIDER_RECONCILIATION_CLOCK_SKEW: Duration = Duration::seconds(5);

#[allow(clippy::too_many_arguments)]
pub(super) async fn reconcile_dialog_boolean_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    observed_state_key: &str,
    expected_state_key: &str,
    observed_mismatch_key: &str,
    observed_state: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
    mismatch_error: &str,
    expected_state_for_command_kind: fn(&str) -> Option<bool>,
    extra_provider_state_fields: &[(&str, serde_json::Value)],
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND happened_at <= $3
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(observed_at + PROVIDER_RECONCILIATION_CLOCK_SKEW)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let Some(expected_state) = expected_state_for_command_kind(&command.command_kind) else {
            continue;
        };

        if expected_state != observed_state {
            let provider_state = dialog_boolean_reconciliation_payload(
                provider_chat_id,
                observed_via,
                expected_state_key,
                expected_state,
                observed_mismatch_key,
                observed_state,
                extra_provider_state_fields,
            );
            let result_payload = dialog_boolean_reconciliation_payload(
                provider_chat_id,
                observed_via,
                expected_state_key,
                expected_state,
                observed_mismatch_key,
                observed_state,
                &[
                    ("provider_observed_at", json!(observed_at)),
                    ("mismatch", json!(true)),
                ],
            );
            reconciled.push(
                mark_command_mismatch(
                    pool,
                    &command.command_id,
                    observed_at,
                    provider_state,
                    result_payload,
                    mismatch_error,
                )
                .await?,
            );
            continue;
        }

        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                dialog_boolean_reconciliation_payload(
                    provider_chat_id,
                    observed_via,
                    observed_state_key,
                    observed_state,
                    observed_state_key,
                    observed_state,
                    extra_provider_state_fields,
                ),
                dialog_boolean_reconciliation_payload(
                    provider_chat_id,
                    observed_via,
                    observed_state_key,
                    observed_state,
                    observed_state_key,
                    observed_state,
                    &[("provider_observed_at", json!(observed_at))],
                ),
            )
            .await?,
        );
    }
    Ok(reconciled)
}

pub(super) fn expected_archive_state_for_command_kind(command_kind: &str) -> Option<bool> {
    match command_kind {
        "archive" => Some(true),
        "unarchive" => Some(false),
        _ => None,
    }
}

pub(super) fn expected_marked_as_unread_state_for_command_kind(command_kind: &str) -> Option<bool> {
    match command_kind {
        "mark_unread" => Some(true),
        _ => None,
    }
}

pub(super) fn expected_mute_state_for_command_kind(command_kind: &str) -> Option<bool> {
    match command_kind {
        "mute" => Some(true),
        "unmute" => Some(false),
        _ => None,
    }
}

pub(super) fn expected_pin_state_for_command_kind(command_kind: &str) -> Option<bool> {
    match command_kind {
        "pin" => Some(true),
        "unpin" => Some(false),
        _ => None,
    }
}

fn dialog_boolean_reconciliation_payload(
    provider_chat_id: &str,
    observed_via: &str,
    primary_key: &str,
    primary_value: bool,
    secondary_key: &str,
    secondary_value: bool,
    extra_fields: &[(&str, serde_json::Value)],
) -> serde_json::Value {
    let mut payload = serde_json::Map::from_iter([
        (
            "provider_chat_id".to_owned(),
            serde_json::Value::String(provider_chat_id.to_owned()),
        ),
        (
            "source".to_owned(),
            serde_json::Value::String(observed_via.to_owned()),
        ),
        (
            "observed_via".to_owned(),
            serde_json::Value::String(observed_via.to_owned()),
        ),
        (
            primary_key.to_owned(),
            serde_json::Value::Bool(primary_value),
        ),
    ]);
    if secondary_key != primary_key {
        payload.insert(
            secondary_key.to_owned(),
            serde_json::Value::Bool(secondary_value),
        );
    }
    for (key, value) in extra_fields {
        payload.insert((*key).to_owned(), value.clone());
    }
    serde_json::Value::Object(payload)
}
