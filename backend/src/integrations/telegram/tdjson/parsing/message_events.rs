use serde_json::{Value, json};

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::snapshots::{
    TelegramTdlibMessageContentSnapshot, TelegramTdlibMessageDeleteSnapshot,
    TelegramTdlibMessageEditedSnapshot, TelegramTdlibMessageInteractionInfoSnapshot,
    TelegramTdlibMessagePinnedSnapshot, TelegramTdlibMessageSnapshot,
};

use super::message_parts::tdlib_message_text;
use super::messages::parse_tdlib_message_snapshot;
use super::values::{tdlib_string_id, tdlib_unix_datetime_value};

pub(crate) fn parse_tdlib_new_message_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateNewMessage") {
        return Ok(None);
    }

    let message = event.get("message").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateNewMessage missing `message`".to_owned())
    })?;
    parse_tdlib_message_snapshot(message).map(Some)
}

pub(crate) fn parse_tdlib_message_delete_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageDeleteSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateDeleteMessages") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let message_ids = event
        .get("message_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("updateDeleteMessages missing `message_ids`".to_owned())
        })?;
    let provider_message_ids = message_ids
        .iter()
        .map(|value| {
            value
                .as_i64()
                .map(|id| id.to_string())
                .or_else(|| value.as_str().map(ToOwned::to_owned))
                .ok_or_else(|| {
                    TelegramError::TdlibRuntime(
                        "updateDeleteMessages contains a non-numeric `message_ids` value"
                            .to_owned(),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let is_permanent = event
        .get("is_permanent")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("updateDeleteMessages missing `is_permanent`".to_owned())
        })?;
    let from_cache = event
        .get("from_cache")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(Some(TelegramTdlibMessageDeleteSnapshot {
        provider_chat_id,
        provider_message_ids,
        is_permanent,
        from_cache,
        source_event: "updateDeleteMessages".to_owned(),
        raw: event.clone(),
    }))
}

pub(crate) fn parse_tdlib_message_interaction_info_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageInteractionInfoSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateMessageInteractionInfo") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let provider_message_id = tdlib_string_id(event, "message_id")?;
    let interaction_info = event.get("interaction_info").ok_or_else(|| {
        TelegramError::TdlibRuntime(
            "updateMessageInteractionInfo missing `interaction_info`".to_owned(),
        )
    })?;

    Ok(Some(TelegramTdlibMessageInteractionInfoSnapshot {
        provider_chat_id,
        provider_message_id,
        source_event: "updateMessageInteractionInfo".to_owned(),
        raw: json!({
            "interaction_info": interaction_info,
        }),
    }))
}

pub(crate) fn parse_tdlib_message_content_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageContentSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateMessageContent") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let provider_message_id = tdlib_string_id(event, "message_id")?;
    let new_content = event.get("new_content").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateMessageContent missing `new_content`".to_owned())
    })?;
    let text = tdlib_message_text(&json!({
        "content": new_content,
    }))?;

    Ok(Some(TelegramTdlibMessageContentSnapshot {
        provider_chat_id,
        provider_message_id,
        text,
        new_content: new_content.clone(),
        source_event: "updateMessageContent".to_owned(),
        raw: event.clone(),
    }))
}

pub(crate) fn parse_tdlib_message_edited_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessageEditedSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateMessageEdited") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let provider_message_id = tdlib_string_id(event, "message_id")?;
    let edit_date = event.get("edit_date").ok_or_else(|| {
        TelegramError::TdlibRuntime("updateMessageEdited missing `edit_date`".to_owned())
    })?;
    let edit_timestamp = tdlib_unix_datetime_value(edit_date)?;
    let reply_markup = event.get("reply_markup").cloned();

    Ok(Some(TelegramTdlibMessageEditedSnapshot {
        provider_chat_id,
        provider_message_id,
        edit_timestamp,
        reply_markup,
        source_event: "updateMessageEdited".to_owned(),
        raw: event.clone(),
    }))
}

pub(crate) fn parse_tdlib_message_pinned_snapshot(
    event: &Value,
) -> Result<Option<TelegramTdlibMessagePinnedSnapshot>, TelegramError> {
    if event.get("@type").and_then(Value::as_str) != Some("updateMessageIsPinned") {
        return Ok(None);
    }

    let provider_chat_id = tdlib_string_id(event, "chat_id")?;
    let provider_message_id = tdlib_string_id(event, "message_id")?;
    let is_pinned = event
        .get("is_pinned")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            TelegramError::TdlibRuntime("updateMessageIsPinned missing `is_pinned`".to_owned())
        })?;

    Ok(Some(TelegramTdlibMessagePinnedSnapshot {
        provider_chat_id,
        provider_message_id,
        is_pinned,
        source_event: "updateMessageIsPinned".to_owned(),
        raw: event.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_update_new_message_snapshot() {
        let snapshot = parse_tdlib_new_message_snapshot(&json!({
            "@type": "updateNewMessage",
            "message": {
                "@type": "message",
                "chat_id": -100123,
                "id": 42,
                "date": 1_718_618_400,
                "sender_id": {
                    "@type": "messageSenderUser",
                    "user_id": 777
                },
                "content": {
                    "@type": "messageText",
                    "text": {
                        "@type": "formattedText",
                        "text": "hello",
                        "entities": []
                    }
                }
            }
        }))
        .expect("parse updateNewMessage")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert_eq!(snapshot.sender_id, "user:777");
        assert_eq!(snapshot.text, "hello");
    }

    #[test]
    fn parses_update_delete_messages_snapshot() {
        let snapshot = parse_tdlib_message_delete_snapshot(&json!({
            "@type": "updateDeleteMessages",
            "chat_id": -100123,
            "message_ids": [42, 43],
            "is_permanent": true,
            "from_cache": false
        }))
        .expect("parse updateDeleteMessages")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_ids, vec!["42", "43"]);
        assert!(snapshot.is_permanent);
        assert!(!snapshot.from_cache);
    }

    #[test]
    fn parses_update_message_interaction_info_snapshot() {
        let snapshot = parse_tdlib_message_interaction_info_snapshot(&json!({
            "@type": "updateMessageInteractionInfo",
            "chat_id": -100123,
            "message_id": 42,
            "interaction_info": {
                "@type": "messageInteractionInfo",
                "view_count": 0,
                "forward_count": 0,
                "reactions": {
                    "@type": "messageReactions",
                    "reactions": []
                }
            }
        }))
        .expect("parse updateMessageInteractionInfo")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert_eq!(
            snapshot.raw["interaction_info"]["@type"],
            "messageInteractionInfo"
        );
    }

    #[test]
    fn parses_update_message_content_snapshot() {
        let snapshot = parse_tdlib_message_content_snapshot(&json!({
            "@type": "updateMessageContent",
            "chat_id": -100123,
            "message_id": 42,
            "new_content": {
                "@type": "messageText",
                "text": {
                    "@type": "formattedText",
                    "text": "edited",
                    "entities": []
                }
            }
        }))
        .expect("parse updateMessageContent")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert_eq!(snapshot.text, "edited");
        assert_eq!(snapshot.new_content["@type"], "messageText");
    }

    #[test]
    fn parses_update_message_edited_snapshot() {
        let snapshot = parse_tdlib_message_edited_snapshot(&json!({
            "@type": "updateMessageEdited",
            "chat_id": -100123,
            "message_id": 42,
            "edit_date": 1718618400,
            "reply_markup": {
                "@type": "replyMarkupInlineKeyboard",
                "rows": []
            }
        }))
        .expect("parse updateMessageEdited")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert_eq!(
            snapshot.reply_markup.as_ref().expect("reply markup")["@type"],
            "replyMarkupInlineKeyboard"
        );
    }

    #[test]
    fn parses_update_message_is_pinned_snapshot() {
        let snapshot = parse_tdlib_message_pinned_snapshot(&json!({
            "@type": "updateMessageIsPinned",
            "chat_id": -100123,
            "message_id": 42,
            "is_pinned": true
        }))
        .expect("parse updateMessageIsPinned")
        .expect("snapshot");

        assert_eq!(snapshot.provider_chat_id, "-100123");
        assert_eq!(snapshot.provider_message_id, "42");
        assert!(snapshot.is_pinned);
    }
}
