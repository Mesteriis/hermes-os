use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::models::topics::{NewTelegramTopic, TelegramTopic};
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::integrations::telegram::tdjson::TelegramTdlibTopicUpdateSnapshot;
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

use super::topics::telegram_topic_id;

pub(super) async fn publish_topic_event(
    pool: &Option<PgPool>,
    event_bus: &EventBus,
    account_id: &str,
    snapshot: &TelegramTdlibTopicUpdateSnapshot,
) {
    let Some(pool) = pool else {
        return;
    };

    let store = TelegramStore::new(pool.clone());
    let topic = match upsert_topic_update(&store, account_id, snapshot).await {
        Ok(Some(topic)) => topic,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to project topic update");
            return;
        }
    };

    let Ok(event) = topic_updated_event(account_id, &topic, Utc::now()) else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append topic event");
    }

    let _ = event_bus.broadcast(event);
}

async fn upsert_topic_update(
    store: &TelegramStore,
    account_id: &str,
    snapshot: &TelegramTdlibTopicUpdateSnapshot,
) -> Result<Option<TelegramTopic>, TelegramError> {
    let Some(chat) = store
        .telegram_chat(account_id, &snapshot.provider_chat_id)
        .await?
    else {
        return Ok(None);
    };

    let topic = NewTelegramTopic {
        topic_id: telegram_topic_id(&chat.telegram_chat_id, snapshot.topic.provider_topic_id),
        telegram_chat_id: chat.telegram_chat_id,
        account_id: chat.account_id,
        provider_topic_id: snapshot.topic.provider_topic_id,
        provider_chat_id: snapshot.provider_chat_id.clone(),
        title: snapshot.topic.title.clone(),
        icon_emoji: snapshot.topic.icon_emoji.clone(),
        is_pinned: snapshot.topic.is_pinned,
        is_closed: snapshot.topic.is_closed,
    };

    crate::integrations::telegram::client::topics::upsert_topic(store.pool(), &topic)
        .await
        .map(Some)
}

fn topic_updated_event(
    account_id: &str,
    topic: &TelegramTopic,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, crate::platform::events::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_topic_{}_{}_{}",
            account_id,
            topic.topic_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::TOPIC_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_topic",
            "id": topic.topic_id,
            "telegram_chat_id": topic.telegram_chat_id,
            "provider_chat_id": topic.provider_chat_id,
            "provider_topic_id": topic.provider_topic_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": topic.telegram_chat_id,
        "provider_chat_id": topic.provider_chat_id,
        "provider_topic_id": topic.provider_topic_id,
        "topic_id": topic.topic_id,
        "topic": topic,
        "source": "tdlib.updateForumTopicInfo"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateForumTopicInfo"
    }))
    .build()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn topic_updated_event_contains_sanitized_projection_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let topic = TelegramTopic {
            topic_id: "telegram_topic:v1:test".to_owned(),
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_topic_id: 42,
            provider_chat_id: "-100123".to_owned(),
            title: "Release notes".to_owned(),
            icon_emoji: Some("123456".to_owned()),
            is_pinned: true,
            is_closed: false,
            unread_count: 0,
            last_message_at: None,
            metadata: json!({}),
            created_at: occurred_at,
            updated_at: occurred_at,
        };

        let event = topic_updated_event("acct-1", &topic, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::TOPIC_UPDATED);
        assert_eq!(event.subject["id"], "telegram_topic:v1:test");
        assert_eq!(event.payload["topic"]["title"], "Release notes");
        assert_eq!(event.payload["topic"]["is_pinned"], true);
    }
}
