use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};
use crate::integrations::telegram::runtime::TelegramMediaSendType;

use super::identifiers::safe_path_segment;

pub(crate) fn set_tdlib_parameters_request(
    request: &TelegramQrLoginStartRequest,
    database_directory: &Path,
) -> Result<Value, TelegramError> {
    let api_id = request.required_api_id()?;
    let api_hash = request.required_api_hash()?;
    let database_directory = database_directory.to_string_lossy().into_owned();
    let files_directory = Path::new(&database_directory)
        .join("files")
        .to_string_lossy()
        .into_owned();

    let parameters = json!({
        "use_test_dc": false,
        "database_directory": database_directory,
        "files_directory": files_directory,
        "database_encryption_key": tdlib_database_encryption_key(request),
        "use_file_database": true,
        "use_chat_info_database": true,
        "use_message_database": true,
        "use_secret_chats": false,
        "api_id": api_id,
        "api_hash": api_hash,
        "system_language_code": "en",
        "device_model": "Hermes Hub",
        "system_version": std::env::consts::OS,
        "application_version": env!("CARGO_PKG_VERSION"),
        "enable_storage_optimizer": true,
        "ignore_file_names": false
    });

    Ok(json!({
        "@type": "setTdlibParameters",
        "parameters": parameters,
        "@extra": "hermes-set-tdlib-parameters"
    }))
}

fn tdlib_database_encryption_key(request: &TelegramQrLoginStartRequest) -> String {
    request
        .session_encryption_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| STANDARD.encode(value.as_bytes()))
        .unwrap_or_default()
}

pub(crate) fn tdlib_database_directory(request: &TelegramQrLoginStartRequest) -> PathBuf {
    request
        .tdlib_data_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("docker/data/telegram").join(safe_path_segment(&request.account_id))
        })
}

pub(crate) fn check_database_encryption_key_request(
    request: &TelegramQrLoginStartRequest,
) -> Value {
    json!({
        "@type": "checkDatabaseEncryptionKey",
        "encryption_key": tdlib_database_encryption_key(request),
        "@extra": "hermes-check-database-encryption-key"
    })
}

pub(crate) fn tdlib_load_chats_request(limit: i32, extra: &str) -> Value {
    json!({
        "@type": "loadChats",
        "chat_list": null,
        "limit": tdlib_page_limit(limit),
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_chats_request(limit: i32, extra: &str) -> Value {
    json!({
        "@type": "getChats",
        "chat_list": null,
        "limit": tdlib_page_limit(limit),
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_chat_request(chat_id: i64, extra: &str) -> Value {
    json!({
        "@type": "getChat",
        "chat_id": chat_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_basic_group_request(basic_group_id: i64, extra: &str) -> Value {
    json!({
        "@type": "getBasicGroup",
        "basic_group_id": basic_group_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_basic_group_full_info_request(basic_group_id: i64, extra: &str) -> Value {
    json!({
        "@type": "getBasicGroupFullInfo",
        "basic_group_id": basic_group_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_chat_folder_request(chat_folder_id: i64, extra: &str) -> Value {
    json!({
        "@type": "getChatFolder",
        "chat_folder_id": chat_folder_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_chat_history_request(
    chat_id: i64,
    from_message_id: Option<i64>,
    limit: i32,
    only_local: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "getChatHistory",
        "chat_id": chat_id,
        "from_message_id": from_message_id.unwrap_or(0),
        "offset": 0,
        "limit": tdlib_page_limit(limit),
        "only_local": only_local,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_send_text_message_request(
    chat_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TelegramError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "text must not be empty".to_owned(),
        ));
    }

    Ok(json!({
        "@type": "sendMessage",
        "chat_id": chat_id,
        "input_message_content": {
            "@type": "inputMessageText",
            "text": {
                "@type": "formattedText",
                "text": text,
                "entities": []
            },
            "clear_draft": true
        },
        "@extra": extra.trim()
    }))
}

pub(crate) fn tdlib_send_media_message_request(
    chat_id: i64,
    media_type: TelegramMediaSendType,
    local_path: &str,
    caption: Option<&str>,
    filename: Option<&str>,
    extra: &str,
) -> Result<Value, TelegramError> {
    let local_path = local_path.trim();
    if local_path.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "media local_path must not be empty".to_owned(),
        ));
    }
    let input_file = json!({
        "@type": "inputFileLocal",
        "path": local_path
    });
    let caption = formatted_caption(caption);
    let input_message_content = match media_type {
        TelegramMediaSendType::Photo => json!({
            "@type": "inputMessagePhoto",
            "photo": input_file,
            "thumbnail": null,
            "added_sticker_file_ids": [],
            "width": 0,
            "height": 0,
            "caption": caption,
            "show_caption_above_media": false,
            "self_destruct_type": null,
            "has_spoiler": false
        }),
        TelegramMediaSendType::Video => json!({
            "@type": "inputMessageVideo",
            "video": input_file,
            "thumbnail": null,
            "added_sticker_file_ids": [],
            "duration": 0,
            "width": 0,
            "height": 0,
            "supports_streaming": true,
            "caption": caption,
            "show_caption_above_media": false,
            "self_destruct_type": null,
            "has_spoiler": false
        }),
        TelegramMediaSendType::Document => json!({
            "@type": "inputMessageDocument",
            "document": input_file,
            "thumbnail": null,
            "disable_content_type_detection": false,
            "caption": caption
        }),
        TelegramMediaSendType::Audio => json!({
            "@type": "inputMessageAudio",
            "audio": input_file,
            "album_cover_thumbnail": null,
            "duration": 0,
            "title": filename.unwrap_or_default(),
            "performer": "",
            "caption": caption
        }),
        TelegramMediaSendType::Voice => json!({
            "@type": "inputMessageVoiceNote",
            "voice_note": input_file,
            "duration": 0,
            "waveform": "",
            "caption": caption
        }),
        TelegramMediaSendType::Sticker => json!({
            "@type": "inputMessageSticker",
            "sticker": input_file,
            "thumbnail": null,
            "emoji": "",
            "width": 0,
            "height": 0
        }),
        TelegramMediaSendType::Animation => json!({
            "@type": "inputMessageAnimation",
            "animation": input_file,
            "thumbnail": null,
            "duration": 0,
            "width": 0,
            "height": 0,
            "caption": caption,
            "show_caption_above_media": false,
            "has_spoiler": false
        }),
    };

    Ok(json!({
        "@type": "sendMessage",
        "chat_id": chat_id,
        "input_message_content": input_message_content,
        "@extra": extra.trim()
    }))
}

fn formatted_caption(caption: Option<&str>) -> Value {
    json!({
        "@type": "formattedText",
        "text": caption.unwrap_or_default().trim(),
        "entities": []
    })
}

pub(crate) fn tdlib_download_file_request(file_id: i64, priority: i32, extra: &str) -> Value {
    json!({
        "@type": "downloadFile",
        "file_id": file_id,
        "priority": priority.clamp(1, 32),
        "offset": 0,
        "limit": 0,
        "synchronous": true,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_edit_message_text_request(
    chat_id: i64,
    message_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TelegramError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "edit text must not be empty".to_owned(),
        ));
    }
    Ok(json!({
        "@type": "editMessageText",
        "chat_id": chat_id,
        "message_id": message_id,
        "input_message_content": {
            "@type": "inputMessageText",
            "text": {
                "@type": "formattedText",
                "text": text,
                "entities": []
            },
            "clear_draft": false
        },
        "@extra": extra.trim()
    }))
}

pub(crate) fn tdlib_delete_messages_request(
    chat_id: i64,
    message_ids: &[i64],
    revoke: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "deleteMessages",
        "chat_id": chat_id,
        "message_ids": message_ids,
        "revoke": revoke,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_add_message_reaction_request(
    chat_id: i64,
    message_id: i64,
    reaction_emoji: &str,
    extra: &str,
) -> Value {
    json!({
        "@type": "addMessageReaction",
        "chat_id": chat_id,
        "message_id": message_id,
        "reaction_type": {
            "@type": "reactionTypeEmoji",
            "emoji": reaction_emoji.trim()
        },
        "is_big": false,
        "update_recent_reactions": true,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_remove_message_reaction_request(
    chat_id: i64,
    message_id: i64,
    reaction_emoji: &str,
    extra: &str,
) -> Value {
    json!({
        "@type": "removeMessageReaction",
        "chat_id": chat_id,
        "message_id": message_id,
        "reaction_type": {
            "@type": "reactionTypeEmoji",
            "emoji": reaction_emoji.trim()
        },
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_pin_chat_message_request(
    chat_id: i64,
    message_id: i64,
    disable_notification: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "pinChatMessage",
        "chat_id": chat_id,
        "message_id": message_id,
        "disable_notification": disable_notification,
        "only_for_self": false,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_send_reply_request(
    chat_id: i64,
    reply_to_message_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TelegramError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "reply text must not be empty".to_owned(),
        ));
    }
    Ok(json!({
        "@type": "sendMessage",
        "chat_id": chat_id,
        "reply_to": {
            "@type": "inputMessageReplyToMessage",
            "message_id": reply_to_message_id
        },
        "input_message_content": {
            "@type": "inputMessageText",
            "text": {
                "@type": "formattedText",
                "text": text,
                "entities": []
            },
            "clear_draft": true
        },
        "@extra": extra.trim()
    }))
}

pub(crate) fn tdlib_send_forward_request(
    chat_id: i64,
    from_chat_id: i64,
    message_id: i64,
    extra: &str,
) -> Value {
    json!({
        "@type": "forwardMessages",
        "chat_id": chat_id,
        "from_chat_id": from_chat_id,
        "message_ids": [message_id],
        "options": null,
        "send_copy": false,
        "remove_caption": false,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_unpin_chat_message_request(
    chat_id: i64,
    message_id: i64,
    extra: &str,
) -> Value {
    json!({
        "@type": "unpinChatMessage",
        "chat_id": chat_id,
        "message_id": message_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_toggle_chat_marked_as_unread_request(
    chat_id: i64,
    is_marked_as_unread: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "toggleChatIsMarkedAsUnread",
        "chat_id": chat_id,
        "is_marked_as_unread": is_marked_as_unread,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_view_messages_request(
    chat_id: i64,
    message_ids: &[i64],
    force_read: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "viewMessages",
        "chat_id": chat_id,
        "message_ids": message_ids,
        "source": null,
        "force_read": force_read,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_add_chat_to_list_request(chat_id: i64, archived: bool, extra: &str) -> Value {
    let chat_list_type = if archived {
        "chatListArchive"
    } else {
        "chatListMain"
    };

    json!({
        "@type": "addChatToList",
        "chat_id": chat_id,
        "chat_list": {
            "@type": chat_list_type
        },
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_add_chat_to_folder_request(
    chat_id: i64,
    chat_folder_id: i64,
    extra: &str,
) -> Value {
    json!({
        "@type": "addChatToList",
        "chat_id": chat_id,
        "chat_list": {
            "@type": "chatListFolder",
            "chat_folder_id": chat_folder_id
        },
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_set_chat_mute_request(chat_id: i64, muted: bool, extra: &str) -> Value {
    json!({
        "@type": "setChatNotificationSettings",
        "chat_id": chat_id,
        "notification_settings": {
            "@type": "chatNotificationSettings",
            "use_default_mute_for": !muted,
            "mute_for": if muted { 31_708_800 } else { 0 },
            "use_default_sound": true,
            "sound_id": 0,
            "use_default_show_preview": true,
            "show_preview": true,
            "use_default_mute_stories": true,
            "mute_stories": false,
            "use_default_story_sound": true,
            "story_sound_id": 0,
            "use_default_show_story_poster": true,
            "show_story_poster": true,
            "use_default_disable_pinned_message_notifications": true,
            "disable_pinned_message_notifications": false,
            "use_default_disable_mention_notifications": true,
            "disable_mention_notifications": false
        },
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_join_chat_request(chat_id: i64, extra: &str) -> Value {
    json!({
        "@type": "joinChat",
        "chat_id": chat_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_leave_chat_request(chat_id: i64, extra: &str) -> Value {
    json!({
        "@type": "leaveChat",
        "chat_id": chat_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_search_messages_request(query: &str, limit: i32, extra: &str) -> Value {
    json!({
        "@type": "searchMessages",
        "chat_list": { "@type": "chatListMain" },
        "query": query.trim(),
        "offset_date": 0,
        "offset_chat_id": 0,
        "offset_message_id": 0,
        "limit": tdlib_page_limit(limit),
        "filter": { "@type": "searchMessagesFilterEmpty" },
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_search_chat_messages_request(
    chat_id: i64,
    query: &str,
    limit: i32,
    extra: &str,
) -> Value {
    json!({
        "@type": "searchChatMessages",
        "chat_id": chat_id,
        "query": query.trim(),
        "sender_id": null,
        "from_message_id": 0,
        "offset": 0,
        "limit": tdlib_page_limit(limit),
        "filter": { "@type": "searchMessagesFilterEmpty" },
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_forum_topics_request(chat_id: i64, limit: i32, extra: &str) -> Value {
    json!({
        "@type": "getForumTopics",
        "chat_id": chat_id,
        "query": "",
        "offset_date": 0,
        "offset_message_id": 0,
        "offset_message_thread_id": 0,
        "limit": tdlib_page_limit(limit),
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_create_forum_topic_request(
    chat_id: i64,
    title: &str,
    extra: &str,
) -> Result<Value, TelegramError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "forum topic title must not be empty".to_owned(),
        ));
    }

    Ok(json!({
        "@type": "createForumTopic",
        "chat_id": chat_id,
        "name": title,
        "icon_custom_emoji_id": 0,
        "@extra": extra.trim()
    }))
}

pub(crate) fn tdlib_toggle_forum_topic_is_closed_request(
    chat_id: i64,
    message_thread_id: i64,
    is_closed: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "toggleForumTopicIsClosed",
        "chat_id": chat_id,
        "message_thread_id": message_thread_id,
        "is_closed": is_closed,
        "@extra": extra.trim()
    })
}

fn tdlib_get_supergroup_members_request_with_filter(
    supergroup_id: i64,
    filter_type: &str,
    offset: i32,
    limit: i32,
    extra: &str,
) -> Value {
    json!({
        "@type": "getSupergroupMembers",
        "supergroup_id": supergroup_id,
        "filter": { "@type": filter_type.trim() },
        "offset": offset.max(0),
        "limit": tdlib_page_limit(limit),
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_supergroup_members_request(
    supergroup_id: i64,
    offset: i32,
    limit: i32,
    extra: &str,
) -> Value {
    tdlib_get_supergroup_members_request_with_filter(
        supergroup_id,
        "supergroupMembersFilterRecent",
        offset,
        limit,
        extra,
    )
}

pub(crate) fn tdlib_get_supergroup_administrators_request(
    supergroup_id: i64,
    offset: i32,
    limit: i32,
    extra: &str,
) -> Value {
    tdlib_get_supergroup_members_request_with_filter(
        supergroup_id,
        "supergroupMembersFilterAdministrators",
        offset,
        limit,
        extra,
    )
}

fn tdlib_page_limit(limit: i32) -> i32 {
    limit.clamp(1, 100)
}
