use serde_json::{Value, json};

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::identifiers::telegram_chat_id;
use crate::integrations::telegram::client::store::TelegramStore;

pub(crate) async fn message_snapshot_payload(
    store: &TelegramStore,
    message_id: &str,
    base_payload: Value,
) -> Result<Value, TelegramError> {
    let mut payload = match base_payload {
        Value::Object(map) => map,
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
    Ok(Value::Object(payload))
}
