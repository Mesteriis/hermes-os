#![allow(dead_code)]

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use hermes_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, EmailProviderKind, NewProviderAccount,
    NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::workflows::provider_communication_projection::{
    project_raw_telegram_message, project_raw_whatsapp_web_message,
};
use serde_json::json;
use sqlx::postgres::PgPool;

pub async fn live_consistency_pool(test_name: &str) -> Option<PgPool> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Polygraph Gmail",
            format!("polygraph-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_polygraph_{suffix}_{provider_record_id}");
    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:polygraph:{suffix}:{provider_record_id}"),
                format!("batch-polygraph-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"polygraph_test"})),
        )
        .await
        .expect("raw message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub async fn seed_telegram_message(
    pool: &PgPool,
    suffix: u128,
    sender_id: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_telegram_{suffix}");
    let provider_chat_id = format!("telegram-chat-{suffix}");
    let provider_message_id = format!("telegram-message-{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::TelegramUser,
            "Polygraph Telegram",
            format!("polygraph-telegram-{suffix}"),
        ))
        .await
        .expect("provider account");

    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_polygraph_telegram_{suffix}"),
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:polygraph:telegram:{suffix}"),
                format!("batch-polygraph-telegram-{suffix}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": format!("Polygraph Telegram {suffix}"),
                    "chat_kind": "private",
                    "sender_id": sender_id,
                    "sender_display_name": "Polygraph Telegram Sender",
                    "text": body_text,
                    "delivery_state": "received",
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "source": "polygraph_test",
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw telegram message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_telegram_message(&message_store, &raw)
        .await
        .expect("project telegram message")
        .message_id
}

pub async fn seed_whatsapp_message(
    pool: &PgPool,
    suffix: u128,
    sender_id: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_whatsapp_{suffix}");
    let provider_chat_id = format!("whatsapp-chat-{suffix}");
    let provider_message_id = format!("whatsapp-message-{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::WhatsappWeb,
            "Polygraph WhatsApp",
            format!("polygraph-whatsapp-{suffix}"),
        ))
        .await
        .expect("provider account");

    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_polygraph_whatsapp_{suffix}"),
                &account_id,
                "whatsapp_web_message",
                &provider_message_id,
                format!("sha256:polygraph:whatsapp:{suffix}"),
                format!("batch-polygraph-whatsapp-{suffix}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": format!("Polygraph WhatsApp {suffix}"),
                    "sender_id": sender_id,
                    "sender_display_name": "Polygraph WhatsApp Sender",
                    "text": body_text,
                    "delivery_state": "received",
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "source": "polygraph_test",
                "provider": "whatsapp",
                "provider_kind": "whatsapp_web",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw WhatsApp message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_whatsapp_web_message(&message_store, &raw)
        .await
        .expect("project WhatsApp message")
        .message_id
}
