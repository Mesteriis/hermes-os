use serde_json::{Value, json};

pub(crate) fn redact_secret_material(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    if is_secret_like_key(&key) {
                        (key, Value::String("[redacted]".to_owned()))
                    } else {
                        (key, redact_secret_material(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_material).collect())
        }
        other => other,
    }
}

fn is_secret_like_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "access_token"
            | "refresh_token"
            | "session_key"
            | "session_material"
            | "authorization"
            | "cookie"
            | "token"
            | "secret"
            | "secret_key"
            | "password"
    )
}

pub(crate) fn push_json_string_once(items: &mut Vec<Value>, value: &str) {
    if !items.iter().any(|item| item.as_str() == Some(value)) {
        items.push(Value::String(value.to_owned()));
    }
}

pub(crate) fn whatsapp_local_edit_diff(previous_text: Option<&str>, new_text: &str) -> Value {
    json!({"kind":"local_edit_diff","previous_text":previous_text,"new_text":new_text})
}
