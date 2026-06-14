use serde_json::{Value, json};

use crate::domains::mail::core::{CommunicationIngestionStore, NewRawCommunicationRecord};
use crate::domains::mail::messages::MessageProjectionStore;

use super::super::TELEGRAM_MESSAGE_RECORD_KIND;
use super::super::errors::TelegramError;
use super::super::identifiers::telegram_raw_record_id;
use super::super::models::{
    NewTelegramChat, NewTelegramMessage, TelegramMessageIngestResult, TelegramSyncState,
};
use super::super::projection::project_raw_telegram_message;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub async fn ingest_fixture_message(
        &self,
        message: &NewTelegramMessage,
    ) -> Result<TelegramMessageIngestResult, TelegramError> {
        self.ingest_message_with_runtime(message, "fixture", None)
            .await
    }

    pub(in crate::integrations::telegram::client::messages) async fn ingest_message_with_runtime(
        &self,
        message: &NewTelegramMessage,
        runtime_kind: &str,
        tdlib_raw: Option<Value>,
    ) -> Result<TelegramMessageIngestResult, TelegramError> {
        message.validate_for_runtime(runtime_kind)?;
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = self
            .telegram_provider_account(&communication_store, &message.account_id)
            .await?;

        let chat = NewTelegramChat {
            account_id: message.account_id.clone(),
            provider_chat_id: message.provider_chat_id.clone(),
            chat_kind: message.chat_kind,
            title: message.chat_title.clone(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: Some(message.occurred_at),
            metadata: json!({"runtime": runtime_kind}),
        };
        self.upsert_chat(&chat).await?;

        let mut payload = json!({
            "provider_chat_id": message.provider_chat_id,
            "chat_title": message.chat_title,
            "chat_kind": message.chat_kind.as_str(),
            "sender_id": message.sender_id,
            "sender_display_name": message.sender_display_name,
            "text": message.text,
            "delivery_state": message.delivery_state.as_str(),
        });
        if let (Some(payload), Some(tdlib_raw)) = (payload.as_object_mut(), tdlib_raw) {
            payload.insert("tdlib_raw".to_owned(), tdlib_raw);
        }
        let raw_record_id = telegram_raw_record_id(
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            payload,
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "telegram",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": runtime_kind,
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        let raw = communication_store.record_raw_source(&raw).await?;
        let projected =
            project_raw_telegram_message(&MessageProjectionStore::new(self.pool.clone()), &raw)
                .await?;
        self.refresh_message_intelligence_candidates(&projected.message_id)
            .await?;

        Ok(TelegramMessageIngestResult {
            raw_record_id: raw.raw_record_id,
            message_id: projected.message_id,
        })
    }
}
