use serde_json::{Value, json};

use crate::integrations::telegram::tdjson::TelegramTdlibChatSnapshot;

pub(super) fn tdlib_chat_projection_metadata(
    snapshot: &TelegramTdlibChatSnapshot,
    raw_record_id: &str,
    owner_provider_user_id: &str,
) -> Value {
    let mut metadata = json!({
        "runtime": "tdlib",
        "raw_record_id": raw_record_id,
    });

    if let Some(permissions) = tdlib_chat_permissions_metadata(&snapshot.raw)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        metadata_map.insert("tdlib_permissions".to_owned(), permissions);
    }
    if let Some(avatar) = tdlib_chat_avatar_metadata(&snapshot.raw)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        metadata_map.insert("avatar".to_owned(), avatar);
    }
    if let Some(preview) = tdlib_chat_last_message_preview(&snapshot.raw)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        metadata_map.insert("last_message_preview".to_owned(), Value::String(preview));
    }
    if let Some(marked) = snapshot
        .raw
        .get("is_marked_as_unread")
        .and_then(Value::as_bool)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        metadata_map.insert("is_marked_as_unread".to_owned(), Value::Bool(marked));
    }
    if let Some(settings) = tdlib_notification_settings_metadata(&snapshot.raw)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        let is_muted = settings
            .get("use_default_mute_for")
            .and_then(Value::as_bool)
            .zip(settings.get("mute_for").and_then(Value::as_i64))
            .is_some_and(|(use_default, mute_for)| !use_default && mute_for > 0);
        metadata_map.insert("tdlib_notification_settings".to_owned(), settings);
        metadata_map.insert("is_muted".to_owned(), Value::Bool(is_muted));
    }
    if let Some(positions) = tdlib_chat_positions_metadata(&snapshot.raw)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        let is_archived = positions
            .get("archive")
            .and_then(Value::as_object)
            .and_then(|archive| archive.get("order"))
            .and_then(Value::as_i64)
            .is_some_and(|order| order > 0);
        let is_pinned = ["main", "archive"]
            .into_iter()
            .filter_map(|key| positions.get(key))
            .filter_map(Value::as_object)
            .any(|value| {
                value
                    .get("is_pinned")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            });
        metadata_map.insert("tdlib_chat_positions".to_owned(), positions);
        metadata_map.insert("is_archived".to_owned(), Value::Bool(is_archived));
        metadata_map.insert("is_pinned".to_owned(), Value::Bool(is_pinned));
    }

    let Some(chat_type) = snapshot.raw.get("type").and_then(Value::as_object) else {
        return metadata;
    };
    let tdlib_chat_type = chat_type
        .get("@type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if let Some(private_user_id) = tdlib_private_user_id(chat_type) {
        project_private_chat_metadata(
            &mut metadata,
            tdlib_chat_type,
            &private_user_id,
            owner_provider_user_id,
        );
    }
    if let Some(basic_group_id) = tdlib_basic_group_id(chat_type) {
        project_basic_group_metadata(&mut metadata, tdlib_chat_type, basic_group_id);
    }
    if tdlib_chat_type != "chatTypeSupergroup" {
        return metadata;
    }

    project_supergroup_metadata(&mut metadata, snapshot, chat_type, tdlib_chat_type);
    metadata
}

fn project_private_chat_metadata(
    metadata: &mut Value,
    tdlib_chat_type: &str,
    private_user_id: &str,
    owner_provider_user_id: &str,
) {
    let Some(metadata_map) = metadata.as_object_mut() else {
        return;
    };
    metadata_map.insert(
        "tdlib_private_user_id".to_owned(),
        Value::String(private_user_id.to_owned()),
    );

    if tdlib_chat_type != "chatTypePrivate" {
        return;
    }
    let owner_user_id = normalized_telegram_user_id(owner_provider_user_id);
    if owner_user_id.as_deref() != Some(private_user_id) {
        return;
    }

    metadata_map.insert(
        "tdlib_chat_type".to_owned(),
        Value::String(tdlib_chat_type.to_owned()),
    );
    metadata_map.insert("is_saved_messages".to_owned(), Value::Bool(true));
    metadata_map.insert(
        "saved_messages_source".to_owned(),
        Value::String("tdlib_private_self_chat".to_owned()),
    );
}

fn project_supergroup_metadata(
    metadata: &mut Value,
    snapshot: &TelegramTdlibChatSnapshot,
    chat_type: &serde_json::Map<String, Value>,
    tdlib_chat_type: &str,
) {
    let Some(metadata_map) = metadata.as_object_mut() else {
        return;
    };
    metadata_map.insert(
        "tdlib_chat_type".to_owned(),
        Value::String(tdlib_chat_type.to_owned()),
    );
    metadata_map.insert("is_supergroup".to_owned(), Value::Bool(true));
    metadata_map.insert(
        "is_channel_supergroup".to_owned(),
        Value::Bool(
            chat_type
                .get("is_channel")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ),
    );
    metadata_map.insert(
        "is_forum".to_owned(),
        Value::Bool(
            chat_type
                .get("is_forum")
                .or_else(|| snapshot.raw.get("is_forum"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ),
    );
    if let Some(supergroup_id) = chat_type.get("supergroup_id").and_then(Value::as_i64) {
        metadata_map.insert(
            "tdlib_supergroup_id".to_owned(),
            Value::Number(serde_json::Number::from(supergroup_id)),
        );
    }
}

fn project_basic_group_metadata(metadata: &mut Value, tdlib_chat_type: &str, basic_group_id: i64) {
    let Some(metadata_map) = metadata.as_object_mut() else {
        return;
    };
    metadata_map.insert(
        "tdlib_chat_type".to_owned(),
        Value::String(tdlib_chat_type.to_owned()),
    );
    metadata_map.insert(
        "tdlib_basic_group_id".to_owned(),
        Value::Number(serde_json::Number::from(basic_group_id)),
    );
    metadata_map.insert("is_basic_group".to_owned(), Value::Bool(true));
}

fn tdlib_private_user_id(chat_type: &serde_json::Map<String, Value>) -> Option<String> {
    chat_type
        .get("user_id")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
}

fn tdlib_basic_group_id(chat_type: &serde_json::Map<String, Value>) -> Option<i64> {
    (chat_type.get("@type").and_then(Value::as_str) == Some("chatTypeBasicGroup"))
        .then(|| chat_type.get("basic_group_id").and_then(Value::as_i64))
        .flatten()
}

fn normalized_telegram_user_id(external_account_id: &str) -> Option<String> {
    let value = external_account_id.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.strip_prefix("telegram:").unwrap_or(value).to_owned())
}

fn tdlib_chat_permissions_metadata(raw: &Value) -> Option<Value> {
    let permissions = raw.get("permissions").and_then(Value::as_object)?;
    let mut projected = serde_json::Map::new();

    for key in [
        "can_send_messages",
        "can_send_basic_messages",
        "can_send_audios",
        "can_send_documents",
        "can_send_photos",
        "can_send_videos",
        "can_send_video_notes",
        "can_send_voice_notes",
        "can_send_polls",
        "can_send_other_messages",
        "can_add_web_page_previews",
        "can_change_info",
        "can_invite_users",
        "can_pin_messages",
        "can_manage_topics",
    ] {
        if let Some(value) = permissions.get(key).and_then(Value::as_bool) {
            projected.insert(key.to_owned(), Value::Bool(value));
        }
    }

    if projected.is_empty() {
        None
    } else {
        Some(Value::Object(projected))
    }
}

fn tdlib_chat_avatar_metadata(raw: &Value) -> Option<Value> {
    let photo = raw.get("photo")?.as_object()?;
    let file = ["big", "small"]
        .into_iter()
        .filter_map(|size| photo.get(size))
        .find(|file| {
            file.get("id")
                .and_then(Value::as_i64)
                .is_some_and(|id| id > 0)
        })?;
    let file_id = file.get("id")?.as_i64()?;
    let remote_unique_id = file
        .get("remote")
        .and_then(|remote| remote.get("unique_id"))
        .and_then(Value::as_str)
        .map(str::to_owned);

    Some(json!({
        "source": "tdlib_chat_photo",
        "tdlib_file_id": file_id,
        "remote_unique_id": remote_unique_id,
        "downloaded": file
            .get("local")
            .and_then(|local| local.get("is_downloading_completed"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }))
}

fn tdlib_chat_last_message_preview(raw: &Value) -> Option<String> {
    let content = raw.get("last_message")?.get("content")?;
    let content_type = content.get("@type")?.as_str()?;
    let text = match content_type {
        "messageText" => content.get("text")?.get("text")?.as_str(),
        "messagePhoto" | "messageVideo" | "messageDocument" | "messageAnimation" => {
            content.get("caption")?.get("text")?.as_str()
        }
        _ => None,
    }
    .or_else(|| match content_type {
        "messagePhoto" => Some("Photo"),
        "messageVideo" => Some("Video"),
        "messageDocument" => Some("Document"),
        "messageVoiceNote" => Some("Voice message"),
        "messageSticker" => Some("Sticker"),
        _ => None,
    })?;
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return None;
    }
    let mut characters = normalized.chars();
    let preview = characters.by_ref().take(160).collect::<String>();
    Some(if characters.next().is_some() {
        format!("{preview}…")
    } else {
        preview
    })
}

fn tdlib_notification_settings_metadata(raw: &Value) -> Option<Value> {
    let settings = raw
        .get("notification_settings")
        .and_then(Value::as_object)?;
    let use_default_mute_for = settings.get("use_default_mute_for")?.as_bool()?;
    let mute_for = settings.get("mute_for")?.as_i64()?;
    Some(json!({
        "use_default_mute_for": use_default_mute_for,
        "mute_for": mute_for
    }))
}

fn tdlib_chat_positions_metadata(raw: &Value) -> Option<Value> {
    let positions = raw.get("positions").and_then(Value::as_array)?;
    let mut projected = serde_json::Map::new();
    let mut folder_ids = Vec::new();

    for position in positions {
        let list = position.get("list")?;
        let order = position.get("order").and_then(Value::as_i64).unwrap_or(0);
        let is_pinned = position
            .get("is_pinned")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        match list.get("@type").and_then(Value::as_str) {
            Some("chatListMain") => {
                projected.insert(
                    "main".to_owned(),
                    json!({"order": order, "is_pinned": is_pinned}),
                );
            }
            Some("chatListArchive") => {
                projected.insert(
                    "archive".to_owned(),
                    json!({"order": order, "is_pinned": is_pinned}),
                );
            }
            Some("chatListFolder") => {
                if let Some(folder_id) = list.get("chat_folder_id").and_then(Value::as_i64) {
                    folder_ids.push(Value::Number(folder_id.into()));
                }
            }
            _ => {}
        }
    }

    if !folder_ids.is_empty() {
        projected.insert("folder_ids".to_owned(), Value::Array(folder_ids));
    }

    (!projected.is_empty()).then_some(Value::Object(projected))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::integrations::telegram::client::TelegramChatKind;

    #[test]
    fn tdlib_chat_projection_metadata_preserves_supergroup_identity() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "123456789".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Release Supergroup".to_owned(),
            username: Some("release_team".to_owned()),
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 123456789,
                "type": {
                    "@type": "chatTypeSupergroup",
                    "supergroup_id": 555,
                    "is_channel": false,
                    "is_forum": true
                },
                "title": "Release Supergroup"
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-123", "42");

        assert_eq!(metadata["runtime"], "tdlib");
        assert_eq!(metadata["raw_record_id"], "raw-telegram-chat-123");
        assert_eq!(metadata["tdlib_chat_type"], "chatTypeSupergroup");
        assert_eq!(metadata["tdlib_supergroup_id"], 555);
        assert_eq!(metadata["is_supergroup"], true);
        assert_eq!(metadata["is_channel_supergroup"], false);
        assert_eq!(metadata["is_forum"], true);
    }

    #[test]
    fn tdlib_chat_projection_metadata_preserves_permissions() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "123456789".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Release Group".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 123456789,
                "type": {"@type": "chatTypeBasicGroup"},
                "permissions": {
                    "@type": "chatPermissions",
                    "can_send_basic_messages": true,
                    "can_send_polls": false,
                    "can_invite_users": true,
                    "can_pin_messages": false,
                    "ignored_non_boolean": "yes"
                }
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-123", "42");

        assert_eq!(metadata["runtime"], "tdlib");
        assert_eq!(
            metadata["tdlib_permissions"]["can_send_basic_messages"],
            true
        );
        assert_eq!(metadata["tdlib_permissions"]["can_send_polls"], false);
        assert_eq!(metadata["tdlib_permissions"]["can_invite_users"], true);
        assert_eq!(metadata["tdlib_permissions"]["can_pin_messages"], false);
        assert_eq!(
            metadata["tdlib_permissions"].get("ignored_non_boolean"),
            None
        );
    }

    #[test]
    fn tdlib_chat_projection_metadata_preserves_marked_unread_and_mute_state() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "123456789".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Release Group".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 123456789,
                "type": {"@type": "chatTypeBasicGroup"},
                "is_marked_as_unread": true,
                "notification_settings": {
                    "@type": "chatNotificationSettings",
                    "use_default_mute_for": false,
                    "mute_for": 31708800
                }
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-123", "42");

        assert_eq!(metadata["is_marked_as_unread"], true);
        assert_eq!(metadata["is_muted"], true);
        assert_eq!(
            metadata["tdlib_notification_settings"]["use_default_mute_for"],
            false
        );
        assert_eq!(
            metadata["tdlib_notification_settings"]["mute_for"],
            31_708_800
        );
    }

    #[test]
    fn tdlib_chat_projection_metadata_preserves_positions_and_archive_state() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "123456789".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Release Group".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 123456789,
                "type": {"@type": "chatTypeBasicGroup"},
                "positions": [
                    {
                        "@type": "chatPosition",
                        "list": {"@type": "chatListArchive"},
                        "order": 42,
                        "is_pinned": false
                    },
                    {
                        "@type": "chatPosition",
                        "list": {"@type": "chatListFolder", "chat_folder_id": 7},
                        "order": 99,
                        "is_pinned": true
                    }
                ]
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-123", "42");

        assert_eq!(metadata["is_archived"], true);
        assert_eq!(metadata["is_pinned"], false);
        assert_eq!(metadata["tdlib_chat_positions"]["archive"]["order"], 42);
        assert_eq!(metadata["tdlib_chat_positions"]["folder_ids"][0], 7);
    }

    #[test]
    fn tdlib_chat_projection_metadata_keeps_a_provider_avatar_reference() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "123456789".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Release Group".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 123456789,
                "type": {"@type": "chatTypeBasicGroup"},
                "photo": {
                    "small": {"id": 7},
                    "big": {
                        "id": 42,
                        "local": {"is_downloading_completed": true},
                        "remote": {"unique_id": "avatar-remote-42"}
                    }
                }
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-123", "42");

        assert_eq!(metadata["avatar"]["source"], "tdlib_chat_photo");
        assert_eq!(metadata["avatar"]["tdlib_file_id"], 42);
        assert_eq!(metadata["avatar"]["remote_unique_id"], "avatar-remote-42");
        assert_eq!(metadata["avatar"]["downloaded"], true);
    }

    #[test]
    fn tdlib_chat_projection_metadata_keeps_a_compact_last_message_preview() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "123456789".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Release Group".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 123456789,
                "type": {"@type": "chatTypeBasicGroup"},
                "last_message": {
                    "content": {
                        "@type": "messageText",
                        "text": {"text": "  New\n  release   is ready  "}
                    }
                }
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-123", "42");

        assert_eq!(metadata["last_message_preview"], "New release is ready");
    }

    #[test]
    fn tdlib_chat_projection_metadata_marks_saved_messages_from_owner_private_chat() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "777".to_owned(),
            chat_kind: TelegramChatKind::Private,
            title: "Saved Messages".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 777,
                "type": {
                    "@type": "chatTypePrivate",
                    "user_id": 777
                },
                "title": "Saved Messages"
            }),
        };

        let metadata =
            tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-777", "telegram:777");

        assert_eq!(metadata["runtime"], "tdlib");
        assert_eq!(metadata["raw_record_id"], "raw-telegram-chat-777");
        assert_eq!(metadata["tdlib_chat_type"], "chatTypePrivate");
        assert_eq!(metadata["tdlib_private_user_id"], "777");
        assert_eq!(metadata["is_saved_messages"], true);
        assert_eq!(metadata["saved_messages_source"], "tdlib_private_self_chat");
    }

    #[test]
    fn tdlib_chat_projection_metadata_does_not_mark_other_private_chats_as_saved() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "888".to_owned(),
            chat_kind: TelegramChatKind::Private,
            title: "Alice".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 888,
                "type": {
                    "@type": "chatTypePrivate",
                    "user_id": 888
                },
                "title": "Alice"
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-888", "777");

        assert_eq!(metadata["tdlib_private_user_id"], "888");
        assert_eq!(metadata.get("is_saved_messages"), None);
        assert_eq!(metadata.get("saved_messages_source"), None);
    }

    #[test]
    fn tdlib_chat_projection_metadata_preserves_basic_group_identity() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "999".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Basic Group".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 999,
                "type": {
                    "@type": "chatTypeBasicGroup",
                    "basic_group_id": 321
                },
                "title": "Basic Group"
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-999", "42");

        assert_eq!(metadata["tdlib_chat_type"], "chatTypeBasicGroup");
        assert_eq!(metadata["tdlib_basic_group_id"], 321);
        assert_eq!(metadata["is_basic_group"], true);
    }
}
