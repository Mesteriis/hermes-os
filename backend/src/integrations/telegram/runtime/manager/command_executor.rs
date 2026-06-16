use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::client::lifecycle;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

use super::super::commands::{
    request_actor_delete_message, request_actor_edit_message, request_actor_pin_message,
    request_actor_set_reaction,
};
use super::TelegramRuntimeManager;

/// Processes up to `limit` queued provider-write commands for all accounts with active actors.
///
/// Each command kind maps to a TDLib operation. Commands that succeed are marked `completed`;
/// those that fail are retried up to `max_retries`, then marked `failed`.
/// A `telegram.command.status_changed` event is emitted for every status transition.
pub async fn execute_queued_commands(
    pool: &PgPool,
    runtime: &TelegramRuntimeManager,
    event_bus: &EventBus,
    per_account_limit: i64,
) {
    let account_ids = match runtime.active_account_ids() {
        Ok(ids) => ids,
        Err(error) => {
            tracing::warn!(error = %error, "command executor: failed to list active accounts");
            return;
        }
    };

    for account_id in account_ids {
        execute_account_commands(pool, runtime, event_bus, &account_id, per_account_limit).await;
    }
}

async fn execute_account_commands(
    pool: &PgPool,
    runtime: &TelegramRuntimeManager,
    event_bus: &EventBus,
    account_id: &str,
    limit: i64,
) {
    let commands =
        match lifecycle::list_queued_commands_for_execution(pool, account_id, limit).await {
            Ok(cmds) => cmds,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    account_id = %account_id,
                    "command executor: failed to list queued commands"
                );
                return;
            }
        };

    if commands.is_empty() {
        return;
    }

    let command_tx = match runtime.actor_command_tx(account_id) {
        Ok(Some(tx)) => tx,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(
                error = %error,
                account_id = %account_id,
                "command executor: failed to get actor command channel"
            );
            return;
        }
    };

    for command in commands {
        let result = dispatch_command(&command, command_tx.clone()).await;
        let (new_status, last_error) = match result {
            Ok(()) => ("completed".to_owned(), None),
            Err(ref error) => {
                tracing::warn!(
                    error = %error,
                    command_id = %command.command_id,
                    command_kind = %command.command_kind,
                    "command executor: command failed"
                );
                if command.retry_count + 1 >= command.max_retries {
                    ("failed".to_owned(), Some(error.to_string()))
                } else {
                    let _ = lifecycle::retry_command(pool, &command.command_id).await;
                    emit_command_event(
                        event_bus,
                        pool,
                        &command.account_id,
                        &command.command_id,
                        &command.provider_chat_id,
                        command.provider_message_id.as_deref(),
                        "retrying",
                    )
                    .await;
                    continue;
                }
            }
        };

        let completed_at = if new_status == "completed" {
            Some(Utc::now())
        } else {
            None
        };

        if let Err(error) = lifecycle::update_command_status(
            pool,
            &command.command_id,
            &new_status,
            json!({}),
            last_error.as_deref(),
            completed_at,
        )
        .await
        {
            tracing::warn!(
                error = %error,
                command_id = %command.command_id,
                "command executor: failed to update command status"
            );
        }

        emit_command_event(
            event_bus,
            pool,
            &command.account_id,
            &command.command_id,
            &command.provider_chat_id,
            command.provider_message_id.as_deref(),
            &new_status,
        )
        .await;
    }
}

async fn dispatch_command(
    command: &crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand,
    command_tx: std::sync::mpsc::Sender<super::super::state::TelegramRuntimeCommand>,
) -> Result<(), TelegramError> {
    match command.command_kind.as_str() {
        "edit" => {
            let new_text = command
                .payload
                .get("new_text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    TelegramError::InvalidRequest("edit command missing new_text".to_owned())
                })?
                .to_owned();
            let provider_message_id = command
                .provider_message_id
                .as_deref()
                .ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "edit command missing provider_message_id".to_owned(),
                    )
                })?
                .to_owned();
            request_actor_edit_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id,
                new_text,
                command.command_id.clone(),
            )
            .await
        }
        "delete" => {
            let revoke = command
                .payload
                .get("is_provider_delete")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let provider_message_id = command
                .provider_message_id
                .as_deref()
                .ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "delete command missing provider_message_id".to_owned(),
                    )
                })?
                .to_owned();
            request_actor_delete_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id,
                revoke,
                command.command_id.clone(),
            )
            .await
        }
        "react" => {
            let reaction_emoji = command
                .payload
                .get("reaction_emoji")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    TelegramError::InvalidRequest("react command missing reaction_emoji".to_owned())
                })?
                .to_owned();
            let provider_message_id = command
                .provider_message_id
                .as_deref()
                .ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "react command missing provider_message_id".to_owned(),
                    )
                })?
                .to_owned();
            request_actor_set_reaction(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id,
                reaction_emoji,
                true,
                command.command_id.clone(),
            )
            .await
        }
        "unreact" => {
            let reaction_emoji = command
                .payload
                .get("reaction_emoji")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "unreact command missing reaction_emoji".to_owned(),
                    )
                })?
                .to_owned();
            let provider_message_id = command
                .provider_message_id
                .as_deref()
                .ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "unreact command missing provider_message_id".to_owned(),
                    )
                })?
                .to_owned();
            request_actor_set_reaction(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id,
                reaction_emoji,
                false,
                command.command_id.clone(),
            )
            .await
        }
        "pin" => {
            let provider_message_id = command
                .provider_message_id
                .as_deref()
                .ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "pin command missing provider_message_id".to_owned(),
                    )
                })?
                .to_owned();
            request_actor_pin_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id,
                true,
                command.command_id.clone(),
            )
            .await
        }
        "unpin" => {
            let provider_message_id = command
                .provider_message_id
                .as_deref()
                .ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "unpin command missing provider_message_id".to_owned(),
                    )
                })?
                .to_owned();
            request_actor_pin_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id,
                false,
                command.command_id.clone(),
            )
            .await
        }
        other => Err(TelegramError::InvalidRequest(format!(
            "command executor: unsupported command kind `{other}`"
        ))),
    }
}

async fn emit_command_event(
    event_bus: &EventBus,
    pool: &PgPool,
    account_id: &str,
    command_id: &str,
    provider_chat_id: &str,
    message_id: Option<&str>,
    status: &str,
) {
    let now = Utc::now();
    let event = NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        telegram_event_types::COMMAND_STATUS_CHANGED.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": command_id, "kind": "telegram_command"}),
    )
    .payload(json!({
        "command_id": command_id,
        "account_id": account_id,
        "provider_chat_id": provider_chat_id,
        "message_id": message_id,
        "status": status,
        "source": "command_executor"
    }))
    .build();

    let Ok(event) = event else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "command executor: failed to append event");
    }

    let _ = event_bus.broadcast(event);
}
