use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibTopicSnapshot;

use super::values::{tdlib_i64_value, tdlib_unix_datetime_value};

pub(crate) fn parse_tdlib_topic_list(
    response: &Value,
) -> Result<Vec<TelegramTdlibTopicSnapshot>, TelegramError> {
    let topics = response
        .get("topics")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(
                "TDLib forumTopics response missing `topics` array".to_owned(),
            )
        })?;

    topics.iter().map(parse_forum_topic).collect()
}

fn parse_forum_topic(topic: &Value) -> Result<TelegramTdlibTopicSnapshot, TelegramError> {
    let info = topic
        .get("info")
        .ok_or_else(|| TelegramError::TdlibRuntime("forumTopic missing `info` field".to_owned()))?;

    let provider_topic_id = info
        .get("message_thread_id")
        .map(tdlib_i64_value)
        .transpose()?
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("forumTopicInfo missing `message_thread_id`".to_owned())
        })?;

    let title = info
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("Topic {provider_topic_id}"));

    let icon_emoji = info
        .get("icon")
        .and_then(|icon| icon.get("custom_emoji_id"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && *s != "0")
        .map(ToOwned::to_owned);

    let is_pinned = info
        .get("is_pinned")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let is_closed = info
        .get("is_closed")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let unread_count = topic
        .get("unread_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let last_message_at = topic
        .get("last_message")
        .and_then(|m| m.get("date"))
        .map(tdlib_unix_datetime_value)
        .transpose()?;

    Ok(TelegramTdlibTopicSnapshot {
        provider_topic_id,
        title,
        icon_emoji,
        is_pinned,
        is_closed,
        unread_count,
        last_message_at,
    })
}
