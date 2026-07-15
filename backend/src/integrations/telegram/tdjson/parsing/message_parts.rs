use serde_json::Value;

use crate::integrations::telegram::client::errors::TelegramError;

use super::values::tdlib_string_id;

pub(super) fn tdlib_message_sender(message: &Value) -> Result<(String, String), TelegramError> {
    let sender = message
        .get("sender_id")
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib sender_id is required".to_owned()))?;
    match sender.get("@type").and_then(Value::as_str) {
        Some("messageSenderUser") => {
            let user_id = tdlib_string_id(sender, "user_id")?;
            Ok((
                format!("user:{user_id}"),
                format!("Telegram User {user_id}"),
            ))
        }
        Some("messageSenderChat") => {
            let chat_id = tdlib_string_id(sender, "chat_id")?;
            Ok((
                format!("chat:{chat_id}"),
                format!("Telegram Chat {chat_id}"),
            ))
        }
        Some(other) => Err(TelegramError::TdlibRuntime(format!(
            "unsupported TDLib message sender `{other}`"
        ))),
        None => Err(TelegramError::TdlibRuntime(
            "TDLib sender_id @type is required".to_owned(),
        )),
    }
}

pub(super) fn tdlib_message_text(message: &Value) -> Result<String, TelegramError> {
    let content = message.get("content").ok_or_else(|| {
        TelegramError::TdlibRuntime("TDLib message content is required".to_owned())
    })?;
    let content_type = content
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib content @type is required".to_owned()))?;
    let formatted_text = match content_type {
        "messageText" => content.get("text"),
        "messagePhoto" | "messageVideo" | "messageDocument" | "messageAudio"
        | "messageVoiceNote" => content.get("caption"),
        _ => None,
    };

    let text = formatted_text
        .and_then(|value| value.get("text"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    match content_type {
        "messageText" => text.ok_or_else(|| {
            TelegramError::TdlibRuntime("TDLib text message does not contain text".to_owned())
        }),
        "messagePhoto" | "messageVideo" | "messageDocument" | "messageAudio"
        | "messageVoiceNote" | "messageUnsupported" => Ok(text.unwrap_or_default()),
        _ => Ok(text.unwrap_or_default()),
    }
}
