use serde_json::Value;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::chats::TelegramChatKind;

use super::values::{tdlib_i64_value, tdlib_string_id, tdlib_unix_datetime_value};
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibChatSnapshot;

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
