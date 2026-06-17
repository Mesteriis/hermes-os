use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::{
    TelegramError, TelegramManualSendRequest, TelegramStore,
};
use crate::integrations::telegram::tdjson::TelegramTdlibMessageSnapshot;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

use super::super::commands::{
    request_actor_delete_message, request_actor_edit_message, request_actor_forward,
    request_actor_join_chat, request_actor_leave_chat, request_actor_pin_message,
    request_actor_reply, request_actor_send, request_actor_send_media, request_actor_set_reaction,
    request_actor_toggle_chat_archive, request_actor_toggle_chat_mute,
    request_actor_toggle_chat_unread,
};
use super::TelegramRuntimeManager;
use super::command_executor_media::{emit_media_upload_event, media_send_request};

const RETRY_BASE_DELAY_SECONDS: i64 = 30;
const RETRY_MAX_DELAY_SECONDS: i64 = 15 * 60;
const STALE_EXECUTION_LOCK_SECONDS: i64 = 120;

/// Processes due provider-write commands for active Telegram account actors.
///
/// Actor dispatch does not mean provider success. Commands that only get an ACK
/// from TDLib stay `executing` with `reconciliation_status=awaiting_provider`;
/// `completed` is reserved for provider-observed state or TDLib calls that
/// return a provider message snapshot.
pub async fn execute_queued_commands(
    pool: &PgPool,
    runtime: &TelegramRuntimeManager,
    event_bus: &EventBus,
    per_account_limit: i64,
) {
    let now = Utc::now();
    let stale_before = now - Duration::seconds(STALE_EXECUTION_LOCK_SECONDS);
    match lifecycle::recover_stale_executing_commands(pool, now, stale_before).await {
        Ok(commands) => {
            for command in commands {
                let status = command.status.clone();
                emit_command_event(
                    event_bus,
                    pool,
                    &command,
                    &status,
                    json!({"source": "stale_recovery", "error": command.last_error.clone()}),
                )
                .await;
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "command executor: failed to recover stale commands");
        }
    }

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
    let now = Utc::now();
    let commands =
        match lifecycle::claim_due_commands_for_execution(pool, account_id, now, limit).await {
            Ok(commands) => commands,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    account_id = %account_id,
                    "command executor: failed to claim due commands"
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
        emit_command_event(
            event_bus,
            pool,
            &command,
            "executing",
            json!({"source": "command_executor", "phase": "claimed"}),
        )
        .await;

        let result = dispatch_command(pool, &command, command_tx.clone()).await;
        handle_dispatch_result(pool, event_bus, &command, result).await;
    }
}

enum DispatchOutcome {
    AwaitingProvider,
    ObservedMessage(TelegramTdlibMessageSnapshot),
}

async fn dispatch_command(
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    command_tx: std::sync::mpsc::Sender<super::super::state::TelegramRuntimeCommand>,
) -> Result<DispatchOutcome, TelegramError> {
    match command.command_kind.as_str() {
        "send_text" => {
            let snapshot = request_actor_send(
                command_tx,
                TelegramManualSendRequest {
                    command_id: command.command_id.clone(),
                    account_id: command.account_id.clone(),
                    provider_chat_id: command.provider_chat_id.clone(),
                    text: payload_string(command, "text")?,
                },
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "send_media" => {
            let request = media_send_request(pool, command).await?;
            let snapshot = request_actor_send_media(command_tx, request).await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "reply" => {
            let snapshot = request_actor_reply(
                command_tx,
                command.provider_chat_id.clone(),
                payload_string(command, "reply_to_provider_message_id")?,
                payload_string(command, "text")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "forward" => {
            let snapshot = request_actor_forward(
                command_tx,
                command.provider_chat_id.clone(),
                payload_string(command, "from_provider_chat_id")?,
                payload_string(command, "from_provider_message_id")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::ObservedMessage(snapshot))
        }
        "edit" => {
            request_actor_edit_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, "edit")?,
                payload_string(command, "new_text")?,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "delete" => {
            request_actor_delete_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, "delete")?,
                command
                    .payload
                    .get("is_provider_delete")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "react" | "unreact" => {
            let is_active = command.command_kind == "react";
            request_actor_set_reaction(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, &command.command_kind)?,
                payload_string(command, "reaction_emoji")?,
                is_active,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "pin" | "unpin" => {
            let pin = command.command_kind == "pin";
            request_actor_pin_message(
                command_tx,
                command.provider_chat_id.clone(),
                provider_message_id(command, &command.command_kind)?,
                pin,
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "mark_read" | "mark_unread" => {
            request_actor_toggle_chat_unread(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "mark_unread",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "archive" | "unarchive" => {
            request_actor_toggle_chat_archive(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "archive",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "mute" | "unmute" => {
            request_actor_toggle_chat_mute(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_kind == "mute",
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "join" => {
            request_actor_join_chat(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        "leave" => {
            request_actor_leave_chat(
                command_tx,
                command.provider_chat_id.clone(),
                command.command_id.clone(),
            )
            .await?;
            Ok(DispatchOutcome::AwaitingProvider)
        }
        other => Err(TelegramError::InvalidRequest(format!(
            "command executor: unsupported command kind `{other}`"
        ))),
    }
}

async fn handle_dispatch_result(
    pool: &PgPool,
    event_bus: &EventBus,
    command: &TelegramProviderWriteCommand,
    result: Result<DispatchOutcome, TelegramError>,
) {
    let now = Utc::now();
    match result {
        Ok(DispatchOutcome::ObservedMessage(snapshot)) => {
            let telegram_store = TelegramStore::new(pool.clone());
            let import_batch_id = format!(
                "telegram-command:{}:{}",
                command.account_id,
                command.command_id.trim()
            );
            let projection = match telegram_store
                .ingest_tdlib_message_snapshot(&command.account_id, &snapshot, &import_batch_id)
                .await
            {
                Ok(result) => result,
                Err(error) => {
                    handle_command_error(pool, event_bus, command, error, now).await;
                    return;
                }
            };
            let provider_state = json!({
                "provider_chat_id": snapshot.provider_chat_id,
                "provider_message_id": snapshot.provider_message_id,
                "delivery_state": snapshot.delivery_state.as_str(),
                "observed_via": "tdlib_returned_message",
                "raw_record_id": projection.raw_record_id.clone(),
                "message_id": projection.message_id.clone(),
            });
            let result_payload = json!({
                "provider_chat_id": snapshot.provider_chat_id,
                "provider_message_id": snapshot.provider_message_id,
                "delivery_state": snapshot.delivery_state.as_str(),
                "raw_record_id": projection.raw_record_id.clone(),
                "message_id": projection.message_id.clone(),
            });
            if let Err(error) = lifecycle::mark_command_reconciled(
                pool,
                &command.command_id,
                now,
                provider_state,
                result_payload,
            )
            .await
            {
                tracing::warn!(
                    error = %error,
                    command_id = %command.command_id,
                    "command executor: failed to mark command reconciled"
                );
            }
            let reconciled_event_payload = json!({
                "source": "command_executor",
                "reconciliation_status": "observed",
                "provider_observed_at": now,
                "reconciled_at": now,
            });
            emit_command_event(
                event_bus,
                pool,
                command,
                "completed",
                reconciled_event_payload.clone(),
            )
            .await;
            emit_command_event_type(
                event_bus,
                pool,
                command,
                telegram_event_types::COMMAND_RECONCILED,
                "completed",
                reconciled_event_payload,
            )
            .await;
            if command.command_kind == "send_media" {
                emit_media_upload_event(
                    event_bus,
                    pool,
                    command,
                    telegram_event_types::MEDIA_UPLOAD_COMPLETED,
                    json!({
                        "status": "completed",
                        "provider_message_id": snapshot.provider_message_id,
                        "delivery_state": snapshot.delivery_state.as_str(),
                        "message_id": projection.message_id,
                    }),
                )
                .await;
            }
        }
        Ok(DispatchOutcome::AwaitingProvider) => {
            if let Err(error) = lifecycle::mark_command_awaiting_provider(
                pool,
                &command.command_id,
                now,
                json!({"dispatch": "accepted", "awaiting_provider_observed": true}),
            )
            .await
            {
                tracing::warn!(
                    error = %error,
                    command_id = %command.command_id,
                    "command executor: failed to persist awaiting-provider state"
                );
            }
        }
        Err(error) => {
            handle_command_error(pool, event_bus, command, error, now).await;
        }
    }
}

async fn handle_command_error(
    pool: &PgPool,
    event_bus: &EventBus,
    command: &TelegramProviderWriteCommand,
    error: TelegramError,
    now: chrono::DateTime<Utc>,
) {
    tracing::warn!(
        error = %error,
        command_id = %command.command_id,
        command_kind = %command.command_kind,
        "command executor: command failed"
    );
    if is_dead_letter_error(&error) || command.retry_count >= command.max_retries {
        if let Err(update_error) =
            lifecycle::dead_letter_command(pool, &command.command_id, now, &error.to_string()).await
        {
            tracing::warn!(
                error = %update_error,
                command_id = %command.command_id,
                "command executor: failed to dead-letter command"
            );
        }
        emit_command_event(
            event_bus,
            pool,
            command,
            "dead_letter",
            json!({"source": "command_executor", "error": error.to_string(), "dead_lettered_at": now}),
        )
        .await;
        if command.command_kind == "send_media" {
            emit_media_upload_event(
                event_bus,
                pool,
                command,
                telegram_event_types::MEDIA_UPLOAD_FAILED,
                json!({"status": "dead_letter", "error": error.to_string(), "dead_lettered_at": now}),
            )
            .await;
        }
    } else {
        let next_attempt_at = next_attempt_at(now, command.retry_count);
        if let Err(update_error) = lifecycle::schedule_command_retry(
            pool,
            &command.command_id,
            now,
            next_attempt_at,
            &error.to_string(),
        )
        .await
        {
            tracing::warn!(
                error = %update_error,
                command_id = %command.command_id,
                "command executor: failed to schedule command retry"
            );
        }
        emit_command_event(
            event_bus,
            pool,
            command,
            "retrying",
            json!({
                "source": "command_executor",
                "error": error.to_string(),
                "next_attempt_at": next_attempt_at,
            }),
        )
        .await;
        if command.command_kind == "send_media" {
            emit_media_upload_event(
                event_bus,
                pool,
                command,
                telegram_event_types::MEDIA_UPLOAD_FAILED,
                json!({"status": "retrying", "error": error.to_string(), "next_attempt_at": next_attempt_at}),
            )
            .await;
        }
    }
}

fn payload_string(
    command: &TelegramProviderWriteCommand,
    key: &str,
) -> Result<String, TelegramError> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{} command missing `{key}`",
                command.command_kind
            ))
        })
}

fn provider_message_id(
    command: &TelegramProviderWriteCommand,
    operation: &str,
) -> Result<String, TelegramError> {
    command
        .provider_message_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "{operation} command missing provider_message_id"
            ))
        })
}

fn next_attempt_at(now: chrono::DateTime<Utc>, completed_attempts: i32) -> chrono::DateTime<Utc> {
    let retry_index = completed_attempts.saturating_sub(1).max(0) as u32;
    let factor = 1_i64.checked_shl(retry_index.min(30)).unwrap_or(i64::MAX);
    let delay_seconds = RETRY_BASE_DELAY_SECONDS
        .saturating_mul(factor)
        .min(RETRY_MAX_DELAY_SECONDS);
    now + Duration::seconds(delay_seconds)
}

fn is_dead_letter_error(error: &TelegramError) -> bool {
    matches!(error, TelegramError::InvalidRequest(_))
}

async fn emit_command_event(
    event_bus: &EventBus,
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    status: &str,
    extra_payload: Value,
) {
    emit_command_event_type(
        event_bus,
        pool,
        command,
        telegram_event_types::COMMAND_STATUS_CHANGED,
        status,
        extra_payload,
    )
    .await;
}

async fn emit_command_event_type(
    event_bus: &EventBus,
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    event_type: &str,
    status: &str,
    extra_payload: Value,
) {
    let now = Utc::now();
    let mut payload = json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "provider_chat_id": command.provider_chat_id,
        "message_id": command.provider_message_id,
        "status": status,
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    if let Some(payload_obj) = payload.as_object_mut() {
        payload_obj.insert("payload".to_owned(), extra_payload);
    }

    let event = NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_command"}),
    )
    .payload(payload)
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
