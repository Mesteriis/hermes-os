use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::application::provider_communication_projection::{
    ProviderCommunicationProjectionError, record_and_project_telegram_message,
    record_and_project_whatsapp_web_message,
};
use crate::application::review_inbox::{
    ReviewInboxWorkflowError, refresh_message_decisions_into_review,
    refresh_message_task_candidates_into_review,
};
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramError, TelegramMessageIngestResult, TelegramStore, telegram_chat_id,
};
use crate::integrations::whatsapp::client::{
    NewWhatsappWebMessage, WhatsappWebError, WhatsappWebMessageIngestResult, WhatsappWebStore,
};
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, EventStoreError, NewEventEnvelope};

const AUDIT_ACTOR_ID: &str = "hermes-frontend";

#[derive(Clone)]
pub(crate) struct TelegramFixtureIngestApplicationService {
    pool: PgPool,
    store: TelegramStore,
    event_store: EventStore,
    event_bus: EventBus,
}

impl TelegramFixtureIngestApplicationService {
    pub(crate) fn new(
        pool: PgPool,
        store: TelegramStore,
        event_store: EventStore,
        event_bus: EventBus,
    ) -> Self {
        Self {
            pool,
            store,
            event_store,
            event_bus,
        }
    }

    pub(crate) async fn ingest_message(
        &self,
        request: &NewTelegramMessage,
    ) -> Result<TelegramMessageIngestResult, CommunicationFixtureIngestError> {
        let observed = self.store.ingest_fixture_message(request).await?;
        let projected =
            record_and_project_telegram_message(self.pool.clone(), observed.raw).await?;
        let response = TelegramMessageIngestResult {
            raw_record_id: projected.raw_record_id,
            message_id: projected.message_id,
        };

        self.store
            .recompute_chat_unread_count(&observed.telegram_chat_id)
            .await?;
        let message_ids = vec![response.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;

        let event = build_event(
            telegram_event_types::MESSAGE_CREATED,
            &request.account_id,
            &response.message_id,
            telegram_message_snapshot_payload(
                &self.store,
                &response.message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "delivery_state": request.delivery_state.as_str(),
                    "runtime_kind": "fixture",
                }),
            )
            .await?,
        );
        self.publish_event(event).await?;

        Ok(response)
    }

    async fn publish_event(
        &self,
        event: NewEventEnvelope,
    ) -> Result<(), CommunicationFixtureIngestError> {
        if let Err(error) = self.event_store.append(&event).await {
            tracing::warn!(error = %error, "failed to append fixture ingest event");
            return Err(error.into());
        }
        let _ = self.event_bus.broadcast(event);
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct WhatsappFixtureIngestApplicationService {
    pool: PgPool,
    store: WhatsappWebStore,
}

impl WhatsappFixtureIngestApplicationService {
    pub(crate) fn new(pool: PgPool, store: WhatsappWebStore) -> Self {
        Self { pool, store }
    }

    pub(crate) async fn ingest_message(
        &self,
        request: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        let observed = self.store.ingest_fixture_message(request).await?;
        let projected =
            record_and_project_whatsapp_web_message(self.pool.clone(), observed.raw).await?;
        let message_ids = vec![projected.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;

        Ok(WhatsappWebMessageIngestResult {
            raw_record_id: projected.raw_record_id,
            message_id: projected.message_id,
        })
    }
}

#[derive(Debug, Error)]
pub(crate) enum CommunicationFixtureIngestError {
    #[error(transparent)]
    Telegram(#[from] TelegramError),

    #[error(transparent)]
    Whatsapp(#[from] WhatsappWebError),

    #[error(transparent)]
    Projection(#[from] ProviderCommunicationProjectionError),

    #[error(transparent)]
    Review(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),
}

pub(crate) fn build_event(
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id, "actor_id": AUDIT_ACTOR_ID}),
        json!({"id": subject_id, "kind": "telegram_message"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

pub(crate) async fn telegram_message_snapshot_payload(
    store: &TelegramStore,
    message_id: &str,
    base_payload: serde_json::Value,
) -> Result<serde_json::Value, TelegramError> {
    let mut payload = match base_payload {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };

    if let Some(message) = store.message_by_id(message_id).await? {
        payload.insert("message".to_owned(), json!(message));
        if let Some(provider_chat_id) = message.provider_chat_id.as_deref() {
            let projected_chat = store
                .telegram_chat(&message.account_id, provider_chat_id)
                .await?;
            let resolved_chat_id = projected_chat
                .as_ref()
                .map(|chat| chat.telegram_chat_id.clone())
                .unwrap_or_else(|| telegram_chat_id(&message.account_id, provider_chat_id));
            payload.insert("telegram_chat_id".to_owned(), json!(resolved_chat_id));
            if let Some(chat) = projected_chat {
                payload.insert("chat".to_owned(), json!(chat));
            }
        }
    }

    Ok(serde_json::Value::Object(payload))
}
