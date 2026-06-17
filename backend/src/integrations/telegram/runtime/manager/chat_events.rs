use chrono::Utc;
use sqlx::PgPool;

use crate::integrations::telegram::client::models::TelegramChat;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::{
    TelegramError, TelegramProviderChatPositionUpdate, TelegramStore,
    reconcile_archive_commands_from_provider_state,
    reconcile_marked_as_unread_commands_from_provider_state,
    reconcile_mute_commands_from_provider_state, reconcile_pin_commands_from_provider_state,
};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatMarkedAsUnreadSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatUnreadSnapshot,
};
use crate::platform::events::{EventBus, EventStore};

use super::chat_event_payloads::{
    chat_archived_updated_event, chat_marked_as_unread_updated_event,
    chat_notification_settings_updated_event, chat_pinned_updated_event, chat_unread_updated_event,
};
use super::realtime_events::{
    TelegramRuntimeEventBridgeContext, publish_command_reconciled_events,
};

pub(super) async fn publish_chat_unread_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatUnreadSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let chat = match apply_chat_unread_update(&store, account_id, snapshot).await {
        Ok(Some(chat)) => chat,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project unread update");
            return;
        }
    };

    let Ok(event) = chat_unread_updated_event(account_id, &chat, snapshot, Utc::now()) else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append unread chat event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_marked_as_unread_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatMarkedAsUnreadSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let (chat, reconciled) = match apply_chat_marked_as_unread_update(&store, account_id, snapshot)
        .await
    {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project marked-as-unread update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(pool.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) = chat_marked_as_unread_updated_event(account_id, &chat, snapshot, Utc::now())
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append marked-as-unread chat event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_notification_settings_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let (chat, reconciled) = match apply_chat_notification_settings_update(
        &store, account_id, snapshot,
    )
    .await
    {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project notification settings update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(pool.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let Ok(event) =
        chat_notification_settings_updated_event(account_id, &chat, snapshot, Utc::now())
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append notification settings event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) async fn publish_chat_position_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibChatPositionSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let (chat, reconciled) = match apply_chat_position_update(&store, account_id, snapshot).await {
        Ok(Some(result)) => result,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project chat position update");
            return;
        }
    };

    let context = TelegramRuntimeEventBridgeContext::new(Some(pool.clone()), event_bus.clone());
    for command in reconciled {
        publish_command_reconciled_events(Some(&context), &command, &snapshot.source_event).await;
    }

    let occurred_at = Utc::now();
    let pin_event = if matches!(snapshot.list_kind.as_str(), "main" | "archive") {
        match chat_pinned_updated_event(account_id, &chat, snapshot, occurred_at) {
            Ok(event) => Some(event),
            Err(_) => return,
        }
    } else {
        None
    };
    let Ok(archive_event) = chat_archived_updated_event(account_id, &chat, snapshot, occurred_at)
    else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Some(event) = &pin_event
        && let Err(error) = event_store.append(event).await
    {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append pinned chat event");
    }
    if let Err(error) = event_store.append(&archive_event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append archived chat event");
    }

    if let Some(event) = pin_event {
        let _ = event_bus.broadcast(event);
    }
    let _ = event_bus.broadcast(archive_event);
}

async fn apply_chat_unread_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatUnreadSnapshot,
) -> Result<Option<TelegramChat>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    store
        .apply_provider_unread_counts(
            &chat.telegram_chat_id,
            snapshot.unread_count,
            snapshot.unread_mention_count,
            snapshot.last_read_inbox_message_id.as_deref(),
            &snapshot.source_event,
        )
        .await?;

    store.telegram_chat_by_id(&chat.telegram_chat_id).await
}

async fn apply_chat_marked_as_unread_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatMarkedAsUnreadSnapshot,
) -> Result<Option<(TelegramChat, Vec<TelegramProviderWriteCommand>)>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    store
        .apply_provider_marked_as_unread(
            &chat.telegram_chat_id,
            snapshot.is_marked_as_unread,
            &snapshot.source_event,
        )
        .await?;
    let reconciled = reconcile_marked_as_unread_commands_from_provider_state(
        store.pool(),
        account_id,
        &snapshot.provider_chat_id,
        snapshot.is_marked_as_unread,
        Utc::now(),
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await?;
    let Some(chat) = store.telegram_chat_by_id(&chat.telegram_chat_id).await? else {
        return Ok(None);
    };

    Ok(Some((chat, reconciled)))
}

async fn apply_chat_notification_settings_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
) -> Result<Option<(TelegramChat, Vec<TelegramProviderWriteCommand>)>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    store
        .apply_provider_notification_settings(
            &chat.telegram_chat_id,
            snapshot.use_default_mute_for,
            snapshot.mute_for,
            &snapshot.source_event,
        )
        .await?;
    let reconciled = reconcile_mute_commands_from_provider_state(
        store.pool(),
        account_id,
        &snapshot.provider_chat_id,
        snapshot.use_default_mute_for,
        snapshot.mute_for,
        Utc::now(),
        &format!("tdlib.{}", snapshot.source_event),
    )
    .await?;
    let Some(chat) = store.telegram_chat_by_id(&chat.telegram_chat_id).await? else {
        return Ok(None);
    };

    Ok(Some((chat, reconciled)))
}

async fn apply_chat_position_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibChatPositionSnapshot,
) -> Result<Option<(TelegramChat, Vec<TelegramProviderWriteCommand>)>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    let position = TelegramProviderChatPositionUpdate {
        list_kind: snapshot.list_kind.clone(),
        provider_folder_id: snapshot.provider_folder_id,
        order: snapshot.order,
        is_pinned: snapshot.is_pinned,
        source_event: snapshot.source_event.clone(),
    };
    store
        .apply_provider_chat_position(&chat.telegram_chat_id, &position)
        .await?;

    let observed_at = Utc::now();
    let observed_via = format!("tdlib.{}", snapshot.source_event);
    let mut reconciled = Vec::new();
    match (snapshot.list_kind.as_str(), snapshot.order > 0) {
        ("archive", true) => {
            reconciled.extend(
                reconcile_archive_commands_from_provider_state(
                    store.pool(),
                    account_id,
                    &snapshot.provider_chat_id,
                    true,
                    observed_at,
                    &observed_via,
                )
                .await?,
            );
            reconciled.extend(
                reconcile_pin_commands_from_provider_state(
                    store.pool(),
                    account_id,
                    &snapshot.provider_chat_id,
                    snapshot.is_pinned,
                    observed_at,
                    &observed_via,
                )
                .await?,
            );
        }
        ("main", true) => {
            reconciled.extend(
                reconcile_archive_commands_from_provider_state(
                    store.pool(),
                    account_id,
                    &snapshot.provider_chat_id,
                    false,
                    observed_at,
                    &observed_via,
                )
                .await?,
            );
            reconciled.extend(
                reconcile_pin_commands_from_provider_state(
                    store.pool(),
                    account_id,
                    &snapshot.provider_chat_id,
                    snapshot.is_pinned,
                    observed_at,
                    &observed_via,
                )
                .await?,
            );
        }
        _ => {}
    }
    let Some(chat) = store.telegram_chat_by_id(&chat.telegram_chat_id).await? else {
        return Ok(None);
    };
    Ok(Some((chat, reconciled)))
}
