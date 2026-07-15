use serde_json::{Value, json};

pub(super) fn insert_non_empty(
    metadata: &mut serde_json::Map<String, Value>,
    key: &'static str,
    value: String,
) {
    let value = value.trim();
    if !value.is_empty() {
        metadata.insert(key.to_owned(), json!(value));
    }
}

pub(super) fn insert_optional(
    metadata: &mut serde_json::Map<String, Value>,
    key: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value {
        insert_non_empty(metadata, key, value);
    }
}

pub(super) fn non_empty_optional(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
