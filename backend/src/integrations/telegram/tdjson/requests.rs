use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};

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

fn tdlib_page_limit(limit: i32) -> i32 {
    limit.clamp(1, 100)
}
