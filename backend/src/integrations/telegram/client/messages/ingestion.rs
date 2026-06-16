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
        let chat = self.upsert_chat(&chat).await?;

        let mention_metadata = derive_mention_metadata(&message.text, tdlib_raw.as_ref());
        let mut payload = json!({
            "provider_chat_id": message.provider_chat_id,
            "chat_title": message.chat_title,
            "chat_kind": message.chat_kind.as_str(),
            "sender_id": message.sender_id,
            "sender_display_name": message.sender_display_name,
            "text": message.text,
            "delivery_state": message.delivery_state.as_str(),
            "mention_count": mention_metadata.count,
            "mentions": mention_metadata.mentions,
            "mentions_detected_by": mention_metadata.detected_by,
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
        self.recompute_chat_unread_count(&chat.telegram_chat_id)
            .await?;
        self.refresh_message_intelligence_candidates(&projected.message_id)
            .await?;

        Ok(TelegramMessageIngestResult {
            raw_record_id: raw.raw_record_id,
            message_id: projected.message_id,
        })
    }
}

struct MentionMetadata {
    count: i64,
    mentions: Vec<String>,
    detected_by: &'static str,
}

fn derive_mention_metadata(text: &str, tdlib_raw: Option<&Value>) -> MentionMetadata {
    let text_mentions = extract_text_mentions(text);
    let entity_count = tdlib_raw.map(tdlib_mention_entity_count).unwrap_or(0);

    if entity_count > 0 {
        MentionMetadata {
            count: entity_count,
            mentions: text_mentions,
            detected_by: "tdlib_entities",
        }
    } else {
        MentionMetadata {
            count: i64::try_from(text_mentions.len()).unwrap_or(0),
            mentions: text_mentions,
            detected_by: "text_regex",
        }
    }
}

fn extract_text_mentions(text: &str) -> Vec<String> {
    let mut mentions = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] != '@' {
            index += 1;
            continue;
        }
        let mut end = index + 1;
        while end < chars.len() && is_telegram_mention_char(chars[end]) {
            end += 1;
        }
        if end.saturating_sub(index) >= 3 {
            let mention: String = chars[index..end].iter().collect();
            if !mentions.iter().any(|existing| existing == &mention) {
                mentions.push(mention);
            }
        }
        index = end;
    }
    mentions
}

fn is_telegram_mention_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

fn tdlib_mention_entity_count(raw: &Value) -> i64 {
    tdlib_formatted_text_entities(raw)
        .into_iter()
        .flat_map(|entities| entities.iter())
        .filter(|entity| {
            matches!(
                entity
                    .get("type")
                    .and_then(|value| value.get("@type"))
                    .and_then(Value::as_str),
                Some("textEntityTypeMention" | "textEntityTypeMentionName")
            )
        })
        .count() as i64
}

fn tdlib_formatted_text_entities(raw: &Value) -> Vec<&Vec<Value>> {
    let mut entities = Vec::new();
    if let Some(content) = raw.get("content") {
        for key in ["text", "caption"] {
            if let Some(array) = content
                .get(key)
                .and_then(|value| value.get("entities"))
                .and_then(Value::as_array)
            {
                entities.push(array);
            }
        }
    }
    entities
}
