use serde_json::{Value, json};

use crate::integrations::telegram::client::errors::TelegramError;

fn non_empty_trimmed_text(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("")
        .to_owned()
}

fn bool_field(folder: &Value, key: &str) -> bool {
    folder.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn i64_field(folder: &Value, key: &str) -> i64 {
    folder.get(key).and_then(Value::as_i64).unwrap_or_default()
}

fn unique_chat_ids(values: &[i64]) -> Vec<i64> {
    let mut result = Vec::with_capacity(values.len());
    for value in values {
        if !result.contains(value) {
            result.push(*value);
        }
    }
    result
}

fn folder_chat_ids(folder: &Value, key: &str) -> Vec<i64> {
    let ids = folder
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_i64)
        .collect::<Vec<_>>();
    unique_chat_ids(&ids)
}

pub(crate) fn tdlib_edit_chat_folder_remove_chat_request(
    chat_folder_id: i64,
    chat_id: i64,
    folder: &Value,
    extra: &str,
) -> Result<Value, TelegramError> {
    if folder.get("@type").and_then(Value::as_str) != Some("chatFolder") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib getChatFolder response is missing chatFolder payload".to_owned(),
        ));
    }

    let mut pinned_chat_ids = folder_chat_ids(folder, "pinned_chat_ids");
    pinned_chat_ids.retain(|value| *value != chat_id);

    let mut included_chat_ids = folder_chat_ids(folder, "included_chat_ids");
    included_chat_ids.retain(|value| *value != chat_id);

    let mut excluded_chat_ids = folder_chat_ids(folder, "excluded_chat_ids");
    if !excluded_chat_ids.contains(&chat_id) {
        excluded_chat_ids.push(chat_id);
    }

    Ok(json!({
        "@type": "editChatFolder",
        "chat_folder_id": chat_folder_id,
        "folder": {
            "@type": "chatFolder",
            "name": {
                "@type": "chatFolderName",
                "text": non_empty_trimmed_text(
                    folder
                        .get("name")
                        .and_then(|value| value.get("text"))
                        .and_then(Value::as_str)
                ),
                "animate_custom_emoji": folder
                    .get("name")
                    .and_then(|value| value.get("animate_custom_emoji"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            },
            "icon": {
                "@type": "chatFolderIcon",
                "name": non_empty_trimmed_text(
                    folder
                        .get("icon")
                        .and_then(|value| value.get("name"))
                        .and_then(Value::as_str)
                ),
            },
            "color_id": i64_field(folder, "color_id"),
            "is_shareable": bool_field(folder, "is_shareable"),
            "pinned_chat_ids": pinned_chat_ids,
            "included_chat_ids": included_chat_ids,
            "excluded_chat_ids": excluded_chat_ids,
            "exclude_muted": bool_field(folder, "exclude_muted"),
            "exclude_read": bool_field(folder, "exclude_read"),
            "exclude_archived": bool_field(folder, "exclude_archived"),
            "include_contacts": bool_field(folder, "include_contacts"),
            "include_non_contacts": bool_field(folder, "include_non_contacts"),
            "include_bots": bool_field(folder, "include_bots"),
            "include_groups": bool_field(folder, "include_groups"),
            "include_channels": bool_field(folder, "include_channels"),
        },
        "@extra": extra.trim(),
    }))
}
