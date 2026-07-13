use serde_json::{Value, json};

use super::types::{TdlibMediaKind, TdlibProtocolError};

pub fn send_text(chat_id: i64, text: &str, extra: &str) -> Result<Value, TdlibProtocolError> {
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

pub fn edit_text(
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

pub fn send_media(
    chat_id: i64,
    media_kind: TdlibMediaKind,
    local_path: &str,
    caption: Option<&str>,
    filename: Option<&str>,
    extra: &str,
) -> Result<Value, TdlibProtocolError> {
    let local_path = local_path.trim();
    if local_path.is_empty() {
        return Err(TdlibProtocolError::InvalidCommand(
            "media local_path must not be empty",
        ));
    }
    let input_file = json!({"@type": "inputFileLocal", "path": local_path});
    let caption = formatted_caption(caption);
    let input_message_content = match media_kind {
        TdlibMediaKind::Photo => json!({
            "@type": "inputMessagePhoto", "photo": input_file, "thumbnail": null,
            "added_sticker_file_ids": [], "width": 0, "height": 0, "caption": caption,
            "show_caption_above_media": false, "self_destruct_type": null, "has_spoiler": false
        }),
        TdlibMediaKind::Video => json!({
            "@type": "inputMessageVideo", "video": input_file, "thumbnail": null,
            "added_sticker_file_ids": [], "duration": 0, "width": 0, "height": 0,
            "supports_streaming": true, "caption": caption, "show_caption_above_media": false,
            "self_destruct_type": null, "has_spoiler": false
        }),
        TdlibMediaKind::Document => json!({
            "@type": "inputMessageDocument", "document": input_file, "thumbnail": null,
            "disable_content_type_detection": false, "caption": caption
        }),
        TdlibMediaKind::Audio => json!({
            "@type": "inputMessageAudio", "audio": input_file, "album_cover_thumbnail": null,
            "duration": 0, "title": filename.unwrap_or_default(), "performer": "", "caption": caption
        }),
        TdlibMediaKind::Voice => json!({
            "@type": "inputMessageVoiceNote", "voice_note": input_file, "duration": 0,
            "waveform": "", "caption": caption
        }),
        TdlibMediaKind::Sticker => json!({
            "@type": "inputMessageSticker", "sticker": input_file, "thumbnail": null,
            "emoji": "", "width": 0, "height": 0
        }),
        TdlibMediaKind::Animation => json!({
            "@type": "inputMessageAnimation", "animation": input_file, "thumbnail": null,
            "duration": 0, "width": 0, "height": 0, "caption": caption,
            "show_caption_above_media": false, "has_spoiler": false
        }),
    };

    Ok(json!({
        "@type": "sendMessage", "chat_id": chat_id,
        "input_message_content": input_message_content, "@extra": extra.trim()
    }))
}

pub fn delete_messages(chat_id: i64, message_ids: &[i64], revoke: bool, extra: &str) -> Value {
    json!({
        "@type": "deleteMessages", "chat_id": chat_id, "message_ids": message_ids,
        "revoke": revoke, "@extra": extra.trim()
    })
}

pub fn add_reaction(chat_id: i64, message_id: i64, reaction_emoji: &str, extra: &str) -> Value {
    json!({
        "@type": "addMessageReaction", "chat_id": chat_id, "message_id": message_id,
        "reaction_type": {"@type": "reactionTypeEmoji", "emoji": reaction_emoji.trim()},
        "is_big": false, "update_recent_reactions": true, "@extra": extra.trim()
    })
}

pub fn remove_reaction(chat_id: i64, message_id: i64, reaction_emoji: &str, extra: &str) -> Value {
    json!({
        "@type": "removeMessageReaction", "chat_id": chat_id, "message_id": message_id,
        "reaction_type": {"@type": "reactionTypeEmoji", "emoji": reaction_emoji.trim()},
        "@extra": extra.trim()
    })
}

pub fn pin_message(
    chat_id: i64,
    message_id: i64,
    disable_notification: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "pinChatMessage", "chat_id": chat_id, "message_id": message_id,
        "disable_notification": disable_notification, "only_for_self": false, "@extra": extra.trim()
    })
}

pub fn unpin_message(chat_id: i64, message_id: i64, extra: &str) -> Value {
    json!({
        "@type": "unpinChatMessage", "chat_id": chat_id, "message_id": message_id,
        "@extra": extra.trim()
    })
}

pub fn forward_message(chat_id: i64, from_chat_id: i64, message_id: i64, extra: &str) -> Value {
    json!({
        "@type": "forwardMessages", "chat_id": chat_id, "from_chat_id": from_chat_id,
        "message_ids": [message_id], "options": null, "send_copy": false,
        "remove_caption": false, "@extra": extra.trim()
    })
}

pub fn view_messages(chat_id: i64, message_ids: &[i64], force_read: bool, extra: &str) -> Value {
    json!({
        "@type": "viewMessages", "chat_id": chat_id, "message_ids": message_ids,
        "source": null, "force_read": force_read, "@extra": extra.trim()
    })
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

fn formatted_caption(caption: Option<&str>) -> Value {
    json!({
        "@type": "formattedText", "text": caption.unwrap_or_default().trim(), "entities": []
    })
}
