use chrono::Utc;
use serde_json::json;
use sqlx::Row;
use testkit::context::TestContext;

use super::*;
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramChatKind, TelegramDeliveryState,
};
use crate::vault::CommunicationProviderAccountStore;

async fn seed_runtime_account(pool: &sqlx::PgPool, account_id: &str, external: &str) {
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert_runtime_account(
            account_id,
            "telegram_user",
            "Telegram Runtime Account",
            external,
            json!({}),
        )
        .await
        .expect("seed provider account");
}

#[tokio::test]
async fn publish_message_content_updated_event_records_projection_observation() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-content-runtime";
    let provider_chat_id = "-100content-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = EventBus::new();

    seed_runtime_account(&pool, account_id, "telegram-ext-content").await;

    let store = TelegramStore::new(pool.clone());
    let ingested = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Content Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "before".to_owned(),
            import_batch_id: "telegram-runtime-content".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageContentSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        text: "after".to_owned(),
        new_content: json!({
            "@type": "messageText",
            "text": {"@type": "formattedText", "text": "after"}
        }),
        source_event: "updateMessageContent".to_owned(),
        raw: json!({"@type": "message"}),
    };

    publish_message_content_updated_event(&Some(pool.clone()), &event_bus, account_id, &snapshot)
        .await;

    let projected = store
        .message_by_id(&ingested.message_id)
        .await
        .expect("load projected message")
        .expect("projected message");
    assert_eq!(projected.text, "after");

    let observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'communication_message'
          AND link.entity_id = $1
          AND link.relationship_kind = 'telegram_content_projection_update'
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&ingested.message_id)
    .fetch_all(&pool)
    .await
    .expect("content projection observations");
    assert!(
        observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "COMMUNICATION_MESSAGE"
                && row.get::<serde_json::Value, _>("payload")["previous_body_text"]
                    == json!("before")
                && row.get::<serde_json::Value, _>("payload")["body_text"] == json!("after")
        }),
        "telegram content projection observation must exist"
    );
}

#[tokio::test]
async fn publish_message_edited_event_records_metadata_observation() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-edited-runtime";
    let provider_chat_id = "-100edited-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = EventBus::new();
    let edit_timestamp = Utc::now();

    seed_runtime_account(&pool, account_id, "telegram-ext-edited").await;

    let store = TelegramStore::new(pool.clone());
    let ingested = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Edited Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "hello".to_owned(),
            import_batch_id: "telegram-runtime-edited".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageEditedSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        edit_timestamp,
        reply_markup: Some(json!({
            "@type": "replyMarkupInlineKeyboard",
            "rows": []
        })),
        source_event: "updateMessageEdited".to_owned(),
        raw: json!({"@type": "message"}),
    };

    publish_message_edited_event(&Some(pool.clone()), &event_bus, account_id, &snapshot).await;

    let projected = store
        .message_by_id(&ingested.message_id)
        .await
        .expect("load projected message")
        .expect("projected message");
    assert_eq!(
        projected.metadata["provider_edit_timestamp"],
        json!(edit_timestamp.to_rfc3339())
    );

    let observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'communication_message'
          AND link.entity_id = $1
          AND link.relationship_kind = 'telegram_metadata_update'
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&ingested.message_id)
    .fetch_all(&pool)
    .await
    .expect("metadata observations");
    assert!(
        observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "COMMUNICATION_MESSAGE"
                && row.get::<serde_json::Value, _>("payload")["message_metadata"]
                    ["provider_edit_timestamp"]
                    == json!(edit_timestamp.to_rfc3339())
                && row.get::<serde_json::Value, _>("payload")["message_metadata"]
                    ["last_provider_edit_source"]
                    == json!("updateMessageEdited")
        }),
        "telegram metadata update observation must exist"
    );
}

#[tokio::test]
async fn publish_reaction_changed_event_records_reaction_observations() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-reaction-runtime";
    let provider_chat_id = "-100reaction-runtime";
    let provider_message_id = "42";
    let provider_message_ref = format!("{provider_chat_id}:{provider_message_id}");
    let event_bus = EventBus::new();

    seed_runtime_account(&pool, account_id, "telegram-ext-reaction").await;

    let store = TelegramStore::new(pool.clone());
    let ingested = store
        .ingest_fixture_message(&NewTelegramMessage {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_ref.clone(),
            chat_kind: TelegramChatKind::Private,
            chat_title: "Reaction Runtime Chat".to_owned(),
            sender_id: "user:777".to_owned(),
            sender_display_name: "Alice".to_owned(),
            text: "hello".to_owned(),
            import_batch_id: "telegram-runtime-reactions".to_owned(),
            occurred_at: Utc::now(),
            delivery_state: TelegramDeliveryState::Received,
        })
        .await
        .expect("ingest fixture message");

    let snapshot = TelegramTdlibMessageInteractionInfoSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        provider_message_id: provider_message_id.to_owned(),
        source_event: "updateMessageInteractionInfo".to_owned(),
        raw: json!({
            "@type": "message",
            "interaction_info": {
                "@type": "messageInteractionInfo",
                "reactions": {
                    "@type": "messageReactions",
                    "reactions": [
                        {
                            "@type": "messageReaction",
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "total_count": 1,
                            "is_chosen": true
                        }
                    ],
                    "recent_reactions": [
                        {
                            "@type": "messageReaction",
                            "sender_id": {
                                "@type": "messageSenderUser",
                                "user_id": 888
                            },
                            "type": {
                                "@type": "reactionTypeEmoji",
                                "emoji": "👍"
                            },
                            "is_outgoing": false
                        }
                    ]
                }
            }
        }),
    };

    publish_reaction_changed_event(&Some(pool.clone()), &event_bus, account_id, &snapshot).await;

    let reaction_rows = sqlx::query(
        r#"
        SELECT reaction_id, sender_id, reaction_emoji, is_active
        FROM telegram_message_reactions
        WHERE message_id = $1
        ORDER BY sender_id ASC, reaction_emoji ASC
        "#,
    )
    .bind(&ingested.message_id)
    .fetch_all(&pool)
    .await
    .expect("reaction rows");
    assert_eq!(reaction_rows.len(), 1);
    let reaction_id = reaction_rows[0].get::<String, _>("reaction_id");
    assert_eq!(reaction_rows[0].get::<String, _>("sender_id"), "user:888");
    assert_eq!(reaction_rows[0].get::<String, _>("reaction_emoji"), "👍");
    assert!(reaction_rows[0].get::<bool, _>("is_active"));

    let reaction_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'message_reaction'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&reaction_id)
    .fetch_all(&pool)
    .await
    .expect("reaction observations");
    assert!(
        reaction_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_MESSAGE_REACTION"
                && row.get::<String, _>("relationship_kind") == "provider_sync_activate"
                && row.get::<serde_json::Value, _>("payload")["sender_id"] == json!("user:888")
                && row.get::<serde_json::Value, _>("payload")["reaction_emoji"] == json!("👍")
        }),
        "provider_sync_activate reaction observation must exist"
    );

    let metadata_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'communication_message'
          AND link.entity_id = $1
          AND link.relationship_kind = 'telegram_metadata_update'
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&ingested.message_id)
    .fetch_all(&pool)
    .await
    .expect("reaction summary metadata observations");
    assert!(
        metadata_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "COMMUNICATION_MESSAGE"
                && row.get::<serde_json::Value, _>("payload")["message_metadata"]
                    ["reaction_summary"]
                    .is_object()
        }),
        "reaction summary metadata observation must exist"
    );
}
