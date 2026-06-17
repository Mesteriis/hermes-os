use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::tdjson::TelegramTdlibTypingSnapshot;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

use super::super::state::TelegramRuntimeEvent;
use super::chat_events::{
    publish_chat_marked_as_unread_event, publish_chat_notification_settings_event,
    publish_chat_position_event, publish_chat_unread_event,
};
use super::message_events::{
    publish_message_content_updated_event, publish_message_created_event,
    publish_message_deleted_event, publish_message_edited_event, publish_message_pinned_event,
    publish_reaction_changed_event,
};
use super::topic_events::publish_topic_event;

#[derive(Clone)]
pub struct TelegramRuntimeEventBridgeContext {
    pub(super) pool: Option<PgPool>,
    pub(super) event_bus: EventBus,
}

impl TelegramRuntimeEventBridgeContext {
    pub(crate) fn new(pool: Option<PgPool>, event_bus: EventBus) -> Self {
        Self { pool, event_bus }
    }
}

pub(super) fn spawn_telegram_runtime_event_bridge(
    pool: Option<PgPool>,
    event_bus: EventBus,
    account_id: String,
    mut runtime_events: UnboundedReceiver<TelegramRuntimeEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = runtime_events.recv().await {
            match event {
                TelegramRuntimeEvent::MessageCreated(snapshot) => {
                    publish_message_created_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::MessageContentUpdated(snapshot) => {
                    publish_message_content_updated_event(
                        &pool,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageEdited(snapshot) => {
                    publish_message_edited_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::MessagePinnedUpdated(snapshot) => {
                    publish_message_pinned_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::MessageDeleted(snapshot) => {
                    publish_message_deleted_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::MessageInteractionInfoUpdated(snapshot) => {
                    publish_reaction_changed_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::TypingChanged(snapshot) => {
                    publish_typing_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::TopicUpdated(snapshot) => {
                    publish_topic_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::ChatUnreadUpdated(snapshot) => {
                    publish_chat_unread_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::ChatMarkedAsUnreadUpdated(snapshot) => {
                    publish_chat_marked_as_unread_event(&pool, &event_bus, &account_id, &snapshot)
                        .await;
                }
                TelegramRuntimeEvent::ChatNotificationSettingsUpdated(snapshot) => {
                    publish_chat_notification_settings_event(
                        &pool,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatPositionUpdated(snapshot) => {
                    publish_chat_position_event(&pool, &event_bus, &account_id, &snapshot).await;
                }
            }
        }
    });
}

pub(super) async fn publish_command_reconciled_events(
    context: Option<&TelegramRuntimeEventBridgeContext>,
    command: &TelegramProviderWriteCommand,
    source: &str,
) {
    let Some(context) = context else {
        return;
    };
    let payload = json!({
        "source": source,
        "reconciliation_status": command.reconciliation_status,
        "provider_observed_at": command.provider_observed_at,
        "reconciled_at": command.reconciled_at,
        "provider_state": command.provider_state,
    });
    publish_command_event(
        context,
        command,
        telegram_event_types::COMMAND_STATUS_CHANGED,
        payload.clone(),
    )
    .await;
    publish_command_event(
        context,
        command,
        telegram_event_types::COMMAND_RECONCILED,
        payload,
    )
    .await;
}

async fn publish_command_event(
    context: &TelegramRuntimeEventBridgeContext,
    command: &TelegramProviderWriteCommand,
    event_type: &str,
    extra_payload: serde_json::Value,
) {
    let now = Utc::now();
    let mut payload = json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "provider_chat_id": command.provider_chat_id,
        "message_id": command.provider_message_id,
        "status": command.status,
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
        format!(
            "evt_telegram_command_{}_{}_{}",
            event_type.replace('.', "_"),
            command.command_id,
            now.timestamp_nanos_opt().unwrap_or(0)
        ),
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

    if let Some(pool) = &context.pool {
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append command reconciliation event");
        }
    }

    let _ = context.event_bus.broadcast(event);
}

async fn publish_typing_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibTypingSnapshot,
) {
    let Ok(event) = typing_changed_event(account_id, snapshot, Utc::now()) else {
        return;
    };

    if let Some(pool) = pool {
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append typing event");
        }
    }

    let _ = event_bus.broadcast(event);
}

fn typing_changed_event(
    account_id: &str,
    snapshot: &TelegramTdlibTypingSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_typing_{}_{}_{}",
            account_id,
            snapshot.provider_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::TYPING_CHANGED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "provider_chat_id": snapshot.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "provider_chat_id": snapshot.provider_chat_id,
        "provider_thread_id": snapshot.provider_thread_id,
        "sender_id": snapshot.sender_id,
        "action": snapshot.action,
        "is_active": snapshot.is_active,
        "source": "tdlib.updateUserChatAction"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateUserChatAction"
    }))
    .build()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn typing_changed_event_contains_sanitized_runtime_projection_payload() {
        let snapshot = TelegramTdlibTypingSnapshot {
            provider_chat_id: "-100123".to_owned(),
            provider_thread_id: Some("42".to_owned()),
            sender_id: "user:777".to_owned(),
            action: "chatActionTyping".to_owned(),
            is_active: true,
        };
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");

        let event = typing_changed_event("acct-1", &snapshot, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::TYPING_CHANGED);
        assert_eq!(event.source["account_id"], "acct-1");
        assert_eq!(event.subject["provider_chat_id"], "-100123");
        assert_eq!(event.payload["provider_thread_id"], "42");
        assert_eq!(event.payload["sender_id"], "user:777");
        assert_eq!(event.payload["is_active"], true);
        assert_eq!(event.provenance["tdlib_event"], "updateUserChatAction");
    }
}
