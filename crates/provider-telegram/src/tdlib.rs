use serde_json::{Value, json};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TdlibProtocolError {
    #[error("invalid Telegram TDLib command: {0}")]
    InvalidCommand(&'static str),
}

pub fn send_text_message(
    chat_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TdlibProtocolError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TdlibProtocolError::InvalidCommand("text must not be empty"));
    }
    Ok(text_message(
        "sendMessage",
        chat_id,
        None,
        text,
        true,
        extra,
    ))
}

pub fn send_reply(
    chat_id: i64,
    reply_to_message_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TdlibProtocolError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TdlibProtocolError::InvalidCommand(
            "reply text must not be empty",
        ));
    }
    Ok(text_message(
        "sendMessage",
        chat_id,
        Some(reply_to_message_id),
        text,
        true,
        extra,
    ))
}

pub fn edit_message_text(
    chat_id: i64,
    message_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TdlibProtocolError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TdlibProtocolError::InvalidCommand(
            "edit text must not be empty",
        ));
    }
    Ok(json!({
        "@type": "editMessageText", "chat_id": chat_id, "message_id": message_id,
        "input_message_content": formatted_text_content(text, false), "@extra": extra.trim()
    }))
}

fn text_message(
    command_type: &str,
    chat_id: i64,
    reply_to_message_id: Option<i64>,
    text: &str,
    clear_draft: bool,
    extra: &str,
) -> Value {
    let mut command = json!({
        "@type": command_type, "chat_id": chat_id,
        "input_message_content": formatted_text_content(text, clear_draft), "@extra": extra.trim()
    });
    if let Some(reply_to_message_id) = reply_to_message_id {
        command["reply_to"] =
            json!({"@type": "inputMessageReplyToMessage", "message_id": reply_to_message_id});
    }
    command
}

fn formatted_text_content(text: &str, clear_draft: bool) -> Value {
    json!({
        "@type": "inputMessageText",
        "text": {"@type": "formattedText", "text": text, "entities": []},
        "clear_draft": clear_draft
    })
}
