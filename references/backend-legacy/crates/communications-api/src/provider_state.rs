use serde_json::Value;

/// Interprets provider metadata without binding the Communications API to a
/// specific provider or persistence implementation.
pub fn observed_read_state(metadata: &Value) -> bool {
    if let Some(labels) = metadata.get("label_ids").and_then(Value::as_array) {
        return !labels.iter().any(|label| label.as_str() == Some("UNREAD"));
    }
    metadata
        .get("is_read")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn observed_starred_state(metadata: &Value) -> Option<bool> {
    metadata.get("starred").and_then(Value::as_bool)
}
