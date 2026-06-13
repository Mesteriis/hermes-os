use serde_json::Value;

use crate::domains::mail::core::StoredRawCommunicationRecord;
use crate::domains::mail::messages::{
    MessageProjectionStore, NewProjectedMessage, ProjectedMessage,
};

use super::TELEGRAM_MESSAGE_RECORD_KIND;
use super::errors::TelegramError;
use super::identifiers::telegram_message_id;
use super::models::TelegramDeliveryState;

pub async fn project_raw_telegram_message(
    store: &MessageProjectionStore,
    raw: &StoredRawCommunicationRecord,
) -> Result<ProjectedMessage, TelegramError> {
    if raw.record_kind != TELEGRAM_MESSAGE_RECORD_KIND {
        return Err(TelegramError::InvalidRequest(
            "raw record kind must be telegram_message".to_owned(),
        ));
    }

    let provider_chat_id = required_payload_string(&raw.payload, "provider_chat_id")?;
    let chat_title = required_payload_string(&raw.payload, "chat_title")?;
    let sender_display_name = required_payload_string(&raw.payload, "sender_display_name")?;
    let text = optional_payload_string(&raw.payload, "text").unwrap_or_default();
    let allow_empty_body_text = text.is_empty() && is_tdlib_raw_payload(raw);
    if text.is_empty() && !allow_empty_body_text {
        return Err(TelegramError::InvalidRequest(
            "payload field `text` is required".to_owned(),
        ));
    }
    let delivery_state =
        TelegramDeliveryState::try_from(required_payload_string(&raw.payload, "delivery_state")?)?;
    let channel_kind = raw
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .unwrap_or("telegram_user");

    let message = NewProjectedMessage {
        message_id: telegram_message_id(&raw.account_id, &raw.provider_record_id),
        raw_record_id: raw.raw_record_id.clone(),
        account_id: raw.account_id.clone(),
        provider_record_id: raw.provider_record_id.clone(),
        subject: chat_title,
        sender: sender_display_name.clone(),
        recipients: vec![provider_chat_id.clone()],
        body_text: text,
        occurred_at: raw.occurred_at,
        channel_kind: channel_kind.to_owned(),
        conversation_id: Some(provider_chat_id),
        sender_display_name: Some(sender_display_name),
        delivery_state: delivery_state.as_message_delivery_state().to_owned(),
        message_metadata: raw.payload.clone(),
    };

    if allow_empty_body_text {
        Ok(store
            .upsert_channel_message_allowing_empty_body_text(&message)
            .await?)
    } else {
        Ok(store.upsert_channel_message(&message).await?)
    }
}

fn required_payload_string(payload: &Value, field: &'static str) -> Result<String, TelegramError> {
    optional_payload_string(payload, field)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!("payload field `{field}` is required"))
        })
}

fn optional_payload_string(payload: &Value, field: &'static str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .map(ToOwned::to_owned)
}

fn is_tdlib_raw_payload(raw: &StoredRawCommunicationRecord) -> bool {
    raw.provenance
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        == Some("tdlib")
        && raw.payload.get("tdlib_raw").is_some()
}
