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
        tdlib_outgoing_delivery_state(message)
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

fn tdlib_outgoing_delivery_state(message: &Value) -> TelegramDeliveryState {
    match message
        .get("sending_state")
        .and_then(|state| state.get("@type"))
        .and_then(Value::as_str)
    {
        Some("messageSendingStatePending") => TelegramDeliveryState::Queued,
        Some("messageSendingStateFailed") => TelegramDeliveryState::SendFailed,
        _ => TelegramDeliveryState::Sent,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn derives_outgoing_delivery_state_from_tdlib_sending_state() {
        let pending = parse_tdlib_message_snapshot(&json!({
            "@type": "message",
            "chat_id": 1,
            "id": -1,
            "date": 1_718_618_400,
            "is_outgoing": true,
            "sending_state": { "@type": "messageSendingStatePending" },
            "sender_id": { "@type": "messageSenderUser", "user_id": 1 },
            "content": { "@type": "messageText", "text": { "@type": "formattedText", "text": "pending", "entities": [] } }
        }))
        .expect("parse pending outgoing message");
        let failed = parse_tdlib_message_snapshot(&json!({
            "@type": "message",
            "chat_id": 1,
            "id": -2,
            "date": 1_718_618_400,
            "is_outgoing": true,
            "sending_state": { "@type": "messageSendingStateFailed" },
            "sender_id": { "@type": "messageSenderUser", "user_id": 1 },
            "content": { "@type": "messageText", "text": { "@type": "formattedText", "text": "failed", "entities": [] } }
        }))
        .expect("parse failed outgoing message");
        let sent = parse_tdlib_message_snapshot(&json!({
            "@type": "message",
            "chat_id": 1,
            "id": 3,
            "date": 1_718_618_400,
            "is_outgoing": true,
            "sender_id": { "@type": "messageSenderUser", "user_id": 1 },
            "content": { "@type": "messageText", "text": { "@type": "formattedText", "text": "sent", "entities": [] } }
        }))
        .expect("parse sent outgoing message");

        assert_eq!(pending.delivery_state, TelegramDeliveryState::Queued);
        assert_eq!(failed.delivery_state, TelegramDeliveryState::SendFailed);
        assert_eq!(sent.delivery_state, TelegramDeliveryState::Sent);
    }
}
