use chrono::{DateTime, Utc};

use super::super::errors::TelegramError;
use super::super::models::TelegramChat;
use super::TelegramStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

impl TelegramStore {
    pub async fn set_chat_metadata_bool(
        &self,
        telegram_chat_id: &str,
        key: &str,
        value: bool,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        metadata.insert(key.to_owned(), serde_json::Value::Bool(value));
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn set_chat_metadata_number(
        &self,
        telegram_chat_id: &str,
        key: &str,
        value: i64,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        metadata.insert(
            key.to_owned(),
            serde_json::Value::Number(serde_json::Number::from(value.max(0))),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn set_chat_last_read_at(
        &self,
        telegram_chat_id: &str,
        last_read_at: Option<DateTime<Utc>>,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        match last_read_at {
            Some(value) => {
                metadata.insert(
                    "last_read_at".to_owned(),
                    serde_json::Value::String(value.to_rfc3339()),
                );
            }
            None => {
                metadata.remove("last_read_at");
            }
        }
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn apply_provider_unread_counts(
        &self,
        telegram_chat_id: &str,
        unread_count: Option<i64>,
        unread_mention_count: Option<i64>,
        last_read_inbox_message_id: Option<&str>,
        source_event: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        if let Some(value) = unread_count {
            metadata.insert(
                "unread_count".to_owned(),
                serde_json::Value::Number(serde_json::Number::from(value.max(0))),
            );
            metadata.insert(
                "provider_unread_count".to_owned(),
                serde_json::Value::Number(serde_json::Number::from(value.max(0))),
            );
        }
        if let Some(value) = unread_mention_count {
            metadata.insert(
                "mention_count".to_owned(),
                serde_json::Value::Number(serde_json::Number::from(value.max(0))),
            );
            metadata.insert(
                "provider_unread_mention_count".to_owned(),
                serde_json::Value::Number(serde_json::Number::from(value.max(0))),
            );
        }
        if let Some(value) = last_read_inbox_message_id {
            metadata.insert(
                "last_read_inbox_provider_message_id".to_owned(),
                serde_json::Value::String(value.to_owned()),
            );
        }
        metadata.insert(
            "unread_count_source".to_owned(),
            serde_json::Value::String(source_event.to_owned()),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn recompute_chat_unread_count(
        &self,
        telegram_chat_id: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let chat = self
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?;
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        let last_read_at = metadata
            .get("last_read_at")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc));
        let (unread_count, mention_count) = self
            .provider_channel_message_store()
            .unread_counts(
                &chat.account_id,
                &chat.provider_chat_id,
                TELEGRAM_CHANNEL_KINDS,
                last_read_at,
            )
            .await?;
        metadata.insert(
            "unread_count".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(unread_count.max(0))),
        );
        metadata.insert(
            "mention_count".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(mention_count.max(0))),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }
}
