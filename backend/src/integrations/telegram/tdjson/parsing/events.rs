use serde_json::Value;

pub(crate) fn authorization_state(event: &Value) -> Option<&Value> {
    match event.get("@type").and_then(Value::as_str) {
        Some("updateAuthorizationState") => event.get("authorization_state"),
        Some(value) if value.starts_with("authorizationState") => Some(event),
        _ => None,
    }
}

pub(crate) fn is_tdlib_parameters_not_specified_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event.get("message").and_then(Value::as_str) == Some("Parameters aren't specified")
}

pub(crate) fn is_tdlib_database_encryption_key_needed_error(event: &Value) -> bool {
    event.get("@type").and_then(Value::as_str) == Some("error")
        && event.get("code").and_then(Value::as_i64) == Some(400)
        && event
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| {
                message.contains("Database encryption key is needed")
                    && message.contains("checkDatabaseEncryptionKey")
            })
}

pub(crate) fn tdlib_error_message(event: &Value) -> Option<String> {
    if event.get("@type").and_then(Value::as_str) != Some("error") {
        return None;
    }

    let code = event
        .get("code")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned());
    let message = event
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("TDLib returned an error");

    Some(format!("TDLib error {code}: {message}"))
}
