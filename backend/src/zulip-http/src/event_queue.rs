use hermes_zulip_api::{ZulipEventQueueV1, ZulipEventV1, ZulipPolledEventV1};
use serde_json::Value;

use crate::{
    ZulipHttpConfigV1,
    command::{request_for_queue_poll, request_for_queue_registration},
    wire::{ZulipHttpErrorV1, execute_value},
};

pub async fn register(config: &ZulipHttpConfigV1) -> Result<ZulipEventQueueV1, ZulipHttpErrorV1> {
    let (_, value) = execute_value(config, request_for_queue_registration(config)?).await?;
    let queue_id = required_string(&value, "queue_id")?;
    let last_event_id = value
        .get("last_event_id")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    (last_event_id >= 0)
        .then_some(())
        .ok_or(ZulipHttpErrorV1::Protocol)?;
    Ok(ZulipEventQueueV1 {
        queue_id,
        last_event_id,
    })
}

#[cfg(test)]
mod attachment_tests {
    use hermes_zulip_api::ZulipAccountV1;
    use serde_json::json;

    use super::{ZulipHttpConfigV1, map_event};

    #[test]
    fn extracts_upload_path_as_metadata_only_attachment() {
        let config = ZulipHttpConfigV1::new(
            ZulipAccountV1 {
                account_id: "account".into(),
                realm_url: "https://zulip.test/".into(),
                bot_email: "bot@zulip.test".into(),
            },
            "secret".into(),
        )
        .expect("config");
        let event = map_event(&config, &json!({
            "id": 1, "type": "message", "message": {
                "id": 2, "stream_id": 3, "subject": "topic", "sender_id": 4,
                "sender_email": "other@zulip.test", "content": "[file](/user_uploads/a/b/report.pdf)"
            }
        })).expect("event");
        let Some(hermes_zulip_api::ZulipEventV1::Message { attachments, .. }) =
            event.observations.first()
        else {
            panic!("message")
        };
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].provider_attachment_id, "a/b/report.pdf");
    }
}

pub async fn poll(
    config: &ZulipHttpConfigV1,
    queue: &ZulipEventQueueV1,
) -> Result<Vec<ZulipPolledEventV1>, ZulipHttpErrorV1> {
    let (_, value) = execute_value(
        config,
        request_for_queue_poll(config, &queue.queue_id, queue.last_event_id)?,
    )
    .await?;
    value
        .get("events")
        .and_then(Value::as_array)
        .ok_or(ZulipHttpErrorV1::Protocol)?
        .iter()
        .map(|event| map_event(config, event))
        .collect()
}

fn map_event(
    config: &ZulipHttpConfigV1,
    value: &Value,
) -> Result<ZulipPolledEventV1, ZulipHttpErrorV1> {
    let event_id = value
        .get("id")
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)
        .ok_or(ZulipHttpErrorV1::Protocol)?;
    let event_type = required_string(value, "type")?;
    let observations = match event_type.as_str() {
        "message" => vec![message_event(config, value, event_id)?],
        "update_message" => vec![message_change(config, value, event_id, true)?],
        "delete_message" => vec![message_change(config, value, event_id, false)?],
        "reaction" => vec![reaction_event(config, value, event_id)?],
        _ => Vec::new(),
    };
    Ok(ZulipPolledEventV1 {
        event_id,
        observations,
    })
}

fn message_event(
    config: &ZulipHttpConfigV1,
    value: &Value,
    event_id: i64,
) -> Result<ZulipEventV1, ZulipHttpErrorV1> {
    let message = value.get("message").ok_or(ZulipHttpErrorV1::Protocol)?;
    let message_id = required_number_string(message, "id")?;
    let conversation = message
        .get("stream_id")
        .and_then(Value::as_i64)
        .map(|stream_id| {
            format!(
                "stream:{stream_id}:{}",
                message.get("subject").and_then(Value::as_str).unwrap_or("")
            )
        })
        .unwrap_or_else(|| {
            format!(
                "direct:{}",
                message
                    .get("recipient_id")
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
            )
        });
    let sender_id = required_number_string(message, "sender_id")?;
    let is_outgoing = message
        .get("sender_email")
        .and_then(Value::as_str)
        .is_some_and(|email| email.eq_ignore_ascii_case(&config.account.bot_email));
    Ok(ZulipEventV1::Message {
        account_id: config.account.account_id.clone(),
        event_id,
        provider_message_id: message_id,
        provider_conversation_id: conversation,
        sender_id,
        is_outgoing,
        content: message
            .get("content")
            .and_then(Value::as_str)
            .map(str::to_owned),
        attachments: message_attachments(message),
    })
}

fn message_attachments(message: &Value) -> Vec<hermes_zulip_api::ZulipAttachmentV1> {
    let paths: Vec<hermes_zulip_api::ZulipAttachmentV1> = message
        .get("attachments")
        .and_then(Value::as_array)
        .map(|attachments| {
            attachments
                .iter()
                .filter_map(attachment_from_value)
                .collect()
        })
        .unwrap_or_else(|| {
            message
                .get("content")
                .and_then(Value::as_str)
                .map(attachment_paths)
                .unwrap_or_default()
                .into_iter()
                .map(attachment_from_path)
                .collect()
        });
    let mut unique: Vec<hermes_zulip_api::ZulipAttachmentV1> = Vec::new();
    for attachment in paths {
        if !attachment.provider_attachment_id.trim().is_empty()
            && !unique
                .iter()
                .any(|existing: &hermes_zulip_api::ZulipAttachmentV1| {
                    existing.provider_attachment_id == attachment.provider_attachment_id
                })
        {
            unique.push(attachment);
        }
    }
    unique
}

fn attachment_from_value(value: &Value) -> Option<hermes_zulip_api::ZulipAttachmentV1> {
    let object = value.as_object()?;
    let id = ["id", "path_id", "url"]
        .iter()
        .find_map(|field| object.get(*field))
        .and_then(value_to_string)?;
    let filename = ["name", "filename"]
        .iter()
        .find_map(|field| object.get(*field))
        .and_then(value_to_string);
    Some(hermes_zulip_api::ZulipAttachmentV1 {
        provider_attachment_id: id,
        filename,
    })
}

fn attachment_paths(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut start = 0;
    while let Some(offset) = content[start..].find("/user_uploads/") {
        let begin = start + offset;
        let tail = &content[begin..];
        let end = tail
            .char_indices()
            .find_map(|(index, character)| {
                matches!(
                    character,
                    '"' | '\'' | ')' | '<' | '>' | ' ' | '\n' | '\r' | '\t'
                )
                .then_some(index)
            })
            .unwrap_or(tail.len());
        let path = tail[..end].to_owned();
        if !paths.contains(&path) {
            paths.push(path);
        }
        start = begin + end.max("/user_uploads/".len());
        if start >= content.len() {
            break;
        }
    }
    paths
}

fn attachment_from_path(path: String) -> hermes_zulip_api::ZulipAttachmentV1 {
    let provider_attachment_id = path
        .strip_prefix("/user_uploads/")
        .unwrap_or(&path)
        .trim_matches('/')
        .to_owned();
    let filename = provider_attachment_id
        .rsplit('/')
        .next()
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned);
    hermes_zulip_api::ZulipAttachmentV1 {
        provider_attachment_id,
        filename,
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) if !value.trim().is_empty() => Some(value.to_owned()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn message_change(
    config: &ZulipHttpConfigV1,
    value: &Value,
    event_id: i64,
    edited: bool,
) -> Result<ZulipEventV1, ZulipHttpErrorV1> {
    let provider_message_id = required_number_string(value, "message_id")?;
    Ok(if edited {
        ZulipEventV1::MessageUpdated {
            account_id: config.account.account_id.clone(),
            event_id,
            provider_message_id,
        }
    } else {
        ZulipEventV1::MessageDeleted {
            account_id: config.account.account_id.clone(),
            event_id,
            provider_message_id,
        }
    })
}

fn reaction_event(
    config: &ZulipHttpConfigV1,
    value: &Value,
    event_id: i64,
) -> Result<ZulipEventV1, ZulipHttpErrorV1> {
    Ok(ZulipEventV1::ReactionChanged {
        account_id: config.account.account_id.clone(),
        event_id,
        provider_message_id: required_number_string(value, "message_id")?,
        actor_id: required_number_string(value, "user_id")?,
    })
}

fn required_string(value: &Value, field: &str) -> Result<String, ZulipHttpErrorV1> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or(ZulipHttpErrorV1::Protocol)
}

fn required_number_string(value: &Value, field: &str) -> Result<String, ZulipHttpErrorV1> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)
        .map(|value| value.to_string())
        .ok_or(ZulipHttpErrorV1::Protocol)
}

#[cfg(test)]
mod tests {
    use hermes_zulip_api::ZulipAccountV1;
    use serde_json::json;

    use super::{ZulipHttpConfigV1, map_event};

    fn config() -> ZulipHttpConfigV1 {
        ZulipHttpConfigV1::new(
            ZulipAccountV1 {
                account_id: "account-1".to_owned(),
                realm_url: "https://zulip.test/".to_owned(),
                bot_email: "bot@zulip.test".to_owned(),
            },
            "secret".to_owned(),
        )
        .expect("config")
    }

    #[test]
    fn maps_message_without_content_and_preserves_outgoing_direction() {
        let event = map_event(
            &config(),
            &json!({
                "id": 7,
                "type": "message",
                "message": {
                    "id": 8,
                    "stream_id": 9,
                    "subject": "topic",
                    "sender_id": 10,
                    "sender_email": "bot@zulip.test"
                }
            }),
        )
        .expect("event");
        assert_eq!(event.event_id, 7);
        let Some(hermes_zulip_api::ZulipEventV1::Message {
            account_id,
            is_outgoing,
            ..
        }) = event.observations.first()
        else {
            panic!("message observation");
        };
        assert_eq!(account_id, "account-1");
        assert!(is_outgoing);
    }

    #[test]
    fn keeps_provider_local_event_as_cursor_only() {
        let event = map_event(&config(), &json!({"id": 7, "type": "realm_emoji"})).expect("event");
        assert!(event.observations.is_empty());
    }
}
