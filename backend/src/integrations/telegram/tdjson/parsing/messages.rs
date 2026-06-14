use chrono::Utc;
use serde_json::Value;

use crate::integrations::telegram::client::{TelegramDeliveryState, TelegramError};

use super::message_parts::{tdlib_message_sender, tdlib_message_text};
use super::values::{tdlib_string_id, tdlib_unix_datetime_value};
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibMessageSnapshot;

pub(crate) fn parse_tdlib_message_list(
    response: &Value,
) -> Result<Vec<TelegramTdlibMessageSnapshot>, TelegramError> {
    let messages = response
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "TDLib getChatHistory response did not include messages".to_owned(),
            )
        })?;

    messages
        .iter()
        .map(parse_tdlib_message_snapshot)
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) fn parse_tdlib_message_snapshot(
    message: &Value,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    if message.get("@type").and_then(Value::as_str) != Some("message") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib message snapshot must have @type=message".to_owned(),
        ));
    }

    let provider_chat_id = tdlib_string_id(message, "chat_id")?;
    let provider_message_id = tdlib_string_id(message, "id")?;
    let (sender_id, sender_display_name) = tdlib_message_sender(message)?;
    let text = tdlib_message_text(message)?;
    let occurred_at = message
        .get("date")
        .map(tdlib_unix_datetime_value)
        .transpose()?
        .unwrap_or_else(Utc::now);
    let delivery_state = if message
        .get("is_outgoing")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        TelegramDeliveryState::Sent
    } else {
        TelegramDeliveryState::Received
    };

    Ok(TelegramTdlibMessageSnapshot {
        provider_chat_id,
        provider_message_id,
        sender_id,
        sender_display_name,
        text,
        occurred_at,
        delivery_state,
        raw: message.clone(),
    })
}
