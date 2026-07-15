use super::errors::TelegramError;
use super::models::chats::TelegramChat;
use super::models::messages::{NewTelegramMessage, TelegramMessage, TelegramObservedMessage};
use super::store::TelegramStore;
use crate::integrations::telegram::client::identifiers::telegram_chat_id;
use serde_json::Value;

/// Narrow fixture/application boundary for Telegram observation and snapshots.
#[derive(Clone)]
pub struct TelegramFixturePort(TelegramStore);

impl TelegramFixturePort {
    pub fn new(store: TelegramStore) -> Self {
        Self(store)
    }

    pub async fn ingest_fixture_message(
        &self,
        message: &NewTelegramMessage,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        self.0.ingest_fixture_message(message).await
    }

    pub async fn message_by_id(
        &self,
        message_id: &str,
    ) -> Result<Option<TelegramMessage>, TelegramError> {
        self.0.message_by_id(message_id).await
    }

    pub async fn telegram_chat(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Result<Option<TelegramChat>, TelegramError> {
        self.0.telegram_chat(account_id, provider_chat_id).await
    }

    pub async fn recompute_chat_unread_count(&self, chat_id: &str) -> Result<Value, TelegramError> {
        self.0.recompute_chat_unread_count(chat_id).await
    }

    pub async fn snapshot_payload(
        &self,
        message_id: &str,
        base_payload: Value,
    ) -> Result<Value, TelegramError> {
        let mut payload = match base_payload {
            Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };
        if let Some(message) = self.message_by_id(message_id).await? {
            payload.insert("message".to_owned(), serde_json::json!(message));
            if let Some(provider_chat_id) = message.provider_chat_id.as_deref() {
                let projected_chat = self
                    .telegram_chat(&message.account_id, provider_chat_id)
                    .await?;
                let resolved_chat_id = projected_chat
                    .as_ref()
                    .map(|chat| chat.telegram_chat_id.clone())
                    .unwrap_or_else(|| telegram_chat_id(&message.account_id, provider_chat_id));
                payload.insert(
                    "telegram_chat_id".to_owned(),
                    serde_json::json!(resolved_chat_id),
                );
                if let Some(chat) = projected_chat {
                    payload.insert("chat".to_owned(), serde_json::json!(chat));
                }
            }
        }
        Ok(Value::Object(payload))
    }
}
