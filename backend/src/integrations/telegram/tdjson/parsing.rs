use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

use crate::integrations::telegram::client::{
    TelegramChatKind, TelegramDeliveryState, TelegramError,
};

use super::snapshots::{
    TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot, TelegramTdlibMessageSnapshot,
};

pub(crate) fn parse_tdlib_chat_ids(response: &Value) -> Result<Vec<i64>, TelegramError> {
    let chat_ids = response
        .get("chat_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "TDLib getChats response did not include chat_ids".to_owned(),
            )
        })?;

    chat_ids
        .iter()
        .map(tdlib_i64_value)
        .collect::<Result<Vec<_>, _>>()
}

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

pub(crate) fn parse_tdlib_chat_snapshot(
    chat: &Value,
) -> Result<TelegramTdlibChatSnapshot, TelegramError> {
    if chat.get("@type").and_then(Value::as_str) != Some("chat") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib chat snapshot must have @type=chat".to_owned(),
        ));
    }

    let provider_chat_id = tdlib_string_id(chat, "id")?;
    let chat_kind = tdlib_chat_kind(chat)?;
    let title = chat
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("Telegram Chat {provider_chat_id}"));
    let username = tdlib_username(chat);
    let last_message_at = chat
        .get("last_message")
        .and_then(|message| message.get("date"))
        .map(tdlib_unix_datetime_value)
        .transpose()?;

    Ok(TelegramTdlibChatSnapshot {
        provider_chat_id,
        chat_kind,
        title,
        username,
        last_message_at,
        raw: chat.clone(),
    })
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

pub(crate) fn parse_tdlib_file_snapshot(
    file: &Value,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    if file.get("@type").and_then(Value::as_str) != Some("file") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib file snapshot must have @type=file".to_owned(),
        ));
    }

    let file_id = file
        .get("id")
        .map(tdlib_i64_value)
        .transpose()?
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib file id is required".to_owned()))?;
    let local = file.get("local").and_then(Value::as_object);
    let remote = file.get("remote").and_then(Value::as_object);
    let local_path = local
        .and_then(|value| value.get("path"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let is_downloading_active = local
        .and_then(|value| value.get("is_downloading_active"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let is_downloading_completed = local
        .and_then(|value| value.get("is_downloading_completed"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let downloaded_size_bytes = local
        .and_then(|value| value.get("downloaded_size"))
        .map(tdlib_i64_value)
        .transpose()?;
    let remote_id = remote
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let remote_unique_id = remote
        .and_then(|value| value.get("unique_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Ok(TelegramTdlibFileSnapshot {
        file_id,
        size_bytes: file.get("size").map(tdlib_i64_value).transpose()?,
        expected_size_bytes: file.get("expected_size").map(tdlib_i64_value).transpose()?,
        local_path,
        is_downloading_active,
        is_downloading_completed,
        downloaded_size_bytes,
        remote_id,
        remote_unique_id,
        raw: file.clone(),
    })
}

fn tdlib_string_id(value: &Value, field: &'static str) -> Result<String, TelegramError> {
    value
        .get(field)
        .map(tdlib_i64_value)
        .transpose()?
        .map(|value| value.to_string())
        .ok_or_else(|| TelegramError::TdlibRuntime(format!("TDLib field `{field}` is required")))
}

fn tdlib_i64_value(value: &Value) -> Result<i64, TelegramError> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib id must be an i64".to_owned()))
}

fn tdlib_unix_datetime_value(value: &Value) -> Result<DateTime<Utc>, TelegramError> {
    let timestamp = value
        .as_i64()
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib date must be an i64".to_owned()))?;
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .ok_or_else(|| TelegramError::TdlibRuntime(format!("invalid TDLib date `{timestamp}`")))
}

fn tdlib_chat_kind(chat: &Value) -> Result<TelegramChatKind, TelegramError> {
    let chat_type = chat
        .get("type")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib chat type is required".to_owned()))?;
    match chat_type {
        "chatTypePrivate" | "chatTypeSecret" => Ok(TelegramChatKind::Private),
        "chatTypeBasicGroup" => Ok(TelegramChatKind::Group),
        "chatTypeSupergroup" => {
            if chat
                .get("type")
                .and_then(|value| value.get("is_channel"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                Ok(TelegramChatKind::Channel)
            } else {
                Ok(TelegramChatKind::Group)
            }
        }
        other => Err(TelegramError::TdlibRuntime(format!(
            "unsupported TDLib chat type `{other}`"
        ))),
    }
}

fn tdlib_username(value: &Value) -> Option<String> {
    value
        .get("username")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            value
                .get("usernames")
                .and_then(|usernames| usernames.get("active_usernames"))
                .and_then(Value::as_array)
                .and_then(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .find(|value| !value.trim().is_empty())
                })
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
}

fn tdlib_message_sender(message: &Value) -> Result<(String, String), TelegramError> {
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

fn tdlib_message_text(message: &Value) -> Result<String, TelegramError> {
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

pub(crate) fn authorization_state(event: &Value) -> Option<&Value> {
    match event.get("@type").and_then(Value::as_str) {
        Some("updateAuthorizationState") => event.get("authorization_state"),
        Some(value) if value.starts_with("authorizationState") => Some(event),
        _ => None,
    }
}

pub(crate) fn is_tdlib_parameters_not_specified_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event.get("message").and_then(Value::as_str) == Some("Parameters aren't specified")
}

pub(crate) fn is_tdlib_database_encryption_key_needed_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| {
                message.contains("Database encryption key is needed")
                    && message.contains("checkDatabaseEncryptionKey")
            })
}

pub(crate) fn tdlib_error_message(event: &Value) -> Option<String> {
    if event.get("@type").and_then(Value::as_str) != Some("error") {
        return None;
    }

    let code = event
        .get("code")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned());
    let message = event
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("TDLib returned an error");

    Some(format!("TDLib error {code}: {message}"))
}
