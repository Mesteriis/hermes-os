use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::lifecycle::{
    reconcile_delete_commands_from_provider_state, reconcile_edit_commands_from_provider_state,
    reconcile_message_pin_commands_from_provider_state, record_provider_delete_observation,
    record_provider_edit_observation,
};
use crate::integrations::telegram::client::{
    TelegramReactionMessageRef, TelegramStore, derive_tdlib_chosen_reaction_emojis,
    derive_tdlib_provider_reactions, derive_tdlib_reaction_summary_metadata,
    reconcile_reaction_commands_from_provider_message_state, sync_provider_reactions,
};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibMessageContentSnapshot, TelegramTdlibMessageDeleteSnapshot,
    TelegramTdlibMessageEditedSnapshot, TelegramTdlibMessageInteractionInfoSnapshot,
    TelegramTdlibMessagePinnedSnapshot, TelegramTdlibMessageSnapshot,
};
use crate::platform::events::EventBus;

use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};

mod envelopes;
mod projection;
#[cfg(test)]
mod tests;

use envelopes::{
    append_and_broadcast, message_created_event, message_deleted_event, message_updated_event,
    reaction_changed_event,
};
use projection::{
    apply_provider_message_content_update, apply_provider_message_edit_metadata,
    observed_edit_timestamp, update_message_reaction_summary,
};

pub(super) async fn publish_message_created_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let import_batch_id = format!(
        "telegram-tdlib-runtime:{}:{}",
        account_id, snapshot.provider_chat_id
    );
    let projection = match store
        .ingest_tdlib_message_snapshot(account_id, snapshot, &import_batch_id)
        .await
    {
        Ok(result) => result,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to ingest message create");
            return;
        }
    };
    let message = match store.message_by_id(&projection.message_id).await {
        Ok(Some(message)) => message,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to load created message");
            return;
        }
    };

    let Ok(event) = message_created_event(account_id, &message, Utc::now()) else {
        return;
    };
    append_and_broadcast(Some(pool.clone()), event_bus, event).await;
}

pub(super) async fn publish_message_deleted_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageDeleteSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let context = TelegramRuntimeEventBridgeContext::new(Some(pool.clone()), event_bus.clone());
    for provider_message_id in &snapshot.provider_message_ids {
        let provider_message_ref = format!("{}:{}", snapshot.provider_chat_id, provider_message_id);
        let message = match store
            .message_by_provider_message_id(account_id, &provider_message_ref)
            .await
        {
            Ok(Some(message)) => message,
            Ok(None) => continue,
            Err(error) => {
                tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load deleted message");
                continue;
            }
        };
        let tombstone = match record_provider_delete_observation(
            pool,
            &message,
            Utc::now(),
            &snapshot.source_event,
            snapshot.is_permanent,
            snapshot.from_cache,
        )
        .await
        {
            Ok(tombstone) => tombstone,
            Err(error) => {
                tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to record provider tombstone");
                continue;
            }
        };
        let reconciled = match reconcile_delete_commands_from_provider_state(
            pool,
            account_id,
            &snapshot.provider_chat_id,
            &provider_message_ref,
            Utc::now(),
            &format!("tdlib.{}", snapshot.source_event),
        )
        .await
        {
            Ok(commands) => commands,
            Err(error) => {
                tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to reconcile delete commands");
                Vec::new()
            }
        };
        for command in reconciled {
            publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event)
                .await;
        }

        let Ok(event) = message_deleted_event(account_id, &message, &tombstone, Utc::now()) else {
            continue;
        };
        append_and_broadcast(Some(pool.clone()), event_bus, event).await;
    }
}

pub(super) async fn publish_message_content_updated_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageContentSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let provider_message_ref = format!(
        "{}:{}",
        snapshot.provider_chat_id, snapshot.provider_message_id
    );
    let Some(message) = (match store
        .message_by_provider_message_id(account_id, &provider_message_ref)
        .await
    {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load edited message");
            return;
        }
    }) else {
        return;
    };

    let previous_text = message.text.clone();
    let observed_at = Utc::now();
    let updated_message = match apply_provider_message_content_update(
        &store,
        &message,
        snapshot,
        observed_at,
    )
    .await
    {
        Ok(Some(message)) => message,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to project message content update");
            return;
        }
    };

    if updated_message.text != previous_text
        && let Err(error) = record_provider_edit_observation(
            pool,
            &updated_message,
            &updated_message.text,
            observed_edit_timestamp(&updated_message, observed_at),
            &snapshot.source_event,
            json!({
                "previous_text": previous_text,
                "new_text": updated_message.text,
                "new_content": snapshot.new_content,
            }),
            json!({
                "provider": "telegram",
                "runtime": "tdlib",
                "source": snapshot.source_event,
            }),
        )
        .await
    {
        tracing::warn!(error = %error, message_id = %updated_message.message_id, "Telegram runtime event bridge: failed to record provider edit version");
    }

    let reconciled = match reconcile_edit_commands_from_provider_state(
        pool,
        account_id,
        &snapshot.provider_chat_id,
        &provider_message_ref,
        &updated_message.text,
        observed_at,
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await
    {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %updated_message.message_id, "Telegram runtime event bridge: failed to reconcile edit commands");
            Vec::new()
        }
    };
    let context = TelegramRuntimeEventBridgeContext::new(Some(pool.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = message_updated_event(
        account_id,
        &updated_message,
        json!({
            "text_changed": updated_message.text != previous_text,
            "provider_edit_timestamp": updated_message.metadata.get("provider_edit_timestamp").cloned(),
            "source": format!("tdlib.{}", snapshot.source_event),
        }),
        observed_at,
    ) else {
        return;
    };
    append_and_broadcast(Some(pool.clone()), event_bus, event).await;
}

pub(super) async fn publish_message_edited_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageEditedSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let provider_message_ref = format!(
        "{}:{}",
        snapshot.provider_chat_id, snapshot.provider_message_id
    );
    let Some(message) = (match store
        .message_by_provider_message_id(account_id, &provider_message_ref)
        .await
    {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load message edit metadata target");
            return;
        }
    }) else {
        return;
    };

    let updated_message = match apply_provider_message_edit_metadata(&store, &message, snapshot)
        .await
    {
        Ok(Some(message)) => message,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to project message edit metadata");
            return;
        }
    };

    let Ok(event) = message_updated_event(
        account_id,
        &updated_message,
        json!({
            "edit_timestamp": snapshot.edit_timestamp,
            "reply_markup_present": snapshot.reply_markup.is_some(),
            "source": format!("tdlib.{}", snapshot.source_event),
        }),
        Utc::now(),
    ) else {
        return;
    };
    append_and_broadcast(Some(pool.clone()), event_bus, event).await;
}

pub(super) async fn publish_message_pinned_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessagePinnedSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let provider_message_ref = format!(
        "{}:{}",
        snapshot.provider_chat_id, snapshot.provider_message_id
    );
    let Some(message) = (match store
        .message_by_provider_message_id(account_id, &provider_message_ref)
        .await
    {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load message pin target");
            return;
        }
    }) else {
        return;
    };

    let observed_at = Utc::now();
    let updated_message = match store
        .apply_message_pinned_state(&message.message_id, snapshot.is_pinned, observed_at)
        .await
    {
        Ok(Some(message)) => message,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to project message pin state");
            return;
        }
    };

    let reconciled = match reconcile_message_pin_commands_from_provider_state(
        pool,
        account_id,
        &snapshot.provider_chat_id,
        &provider_message_ref,
        snapshot.is_pinned,
        observed_at,
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await
    {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %updated_message.message_id, "Telegram runtime event bridge: failed to reconcile message pin commands");
            Vec::new()
        }
    };
    let context = TelegramRuntimeEventBridgeContext::new(Some(pool.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = message_updated_event(
        account_id,
        &updated_message,
        json!({
            "is_pinned": snapshot.is_pinned,
            "source": format!("tdlib.{}", snapshot.source_event),
        }),
        observed_at,
    ) else {
        return;
    };
    append_and_broadcast(Some(pool.clone()), event_bus, event).await;
}

pub(super) async fn publish_reaction_changed_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibMessageInteractionInfoSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let provider_message_ref = format!(
        "{}:{}",
        snapshot.provider_chat_id, snapshot.provider_message_id
    );
    let Some(message) = (match store
        .message_by_provider_message_id(account_id, &provider_message_ref)
        .await
    {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(error = %error, provider_message_id = %provider_message_ref, "Telegram runtime event bridge: failed to load reaction target message");
            return;
        }
    }) else {
        return;
    };

    let provider_reactions = derive_tdlib_provider_reactions(&snapshot.raw);
    let chosen_reactions = derive_tdlib_chosen_reaction_emojis(&snapshot.raw);
    if let Err(error) = sync_provider_reactions(
        pool,
        TelegramReactionMessageRef {
            message_id: &message.message_id,
            account_id: &message.account_id,
            provider_chat_id: &snapshot.provider_chat_id,
            provider_message_id: &provider_message_ref,
        },
        &provider_reactions,
        None,
        &chosen_reactions,
    )
    .await
    {
        tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to sync provider reactions");
        return;
    }

    let updated_message = match update_message_reaction_summary(&store, &message, &snapshot.raw)
        .await
    {
        Ok(Some(message)) => message,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %message.message_id, "Telegram runtime event bridge: failed to update reaction summary metadata");
            return;
        }
    };

    let reconciled = match reconcile_reaction_commands_from_provider_message_state(
        pool,
        account_id,
        &snapshot.provider_chat_id,
        &provider_message_ref,
        &chosen_reactions,
        Utc::now(),
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await
    {
        Ok(commands) => commands,
        Err(error) => {
            tracing::warn!(error = %error, message_id = %updated_message.message_id, "Telegram runtime event bridge: failed to reconcile reaction commands");
            Vec::new()
        }
    };
    let context = TelegramRuntimeEventBridgeContext::new(Some(pool.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = reaction_changed_event(
        account_id,
        &updated_message,
        derive_tdlib_reaction_summary_metadata(&snapshot.raw),
        Utc::now(),
    ) else {
        return;
    };
    append_and_broadcast(Some(pool.clone()), event_bus, event).await;
}
