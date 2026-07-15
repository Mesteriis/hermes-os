use connectrpc::{ConnectError, ErrorCode};
use serde_json::{Value, json};

pub(super) fn parse_json_object(value: &str, field_name: &str) -> Result<Value, ConnectError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(json!({}));
    }
    let parsed: Value = serde_json::from_str(trimmed).map_err(|_| {
        ConnectError::new(
            ErrorCode::InvalidArgument,
            format!("{field_name} must contain valid JSON"),
        )
    })?;
    if !parsed.is_object() {
        return Err(ConnectError::new(
            ErrorCode::InvalidArgument,
            format!("{field_name} must contain a JSON object"),
        ));
    }
    Ok(parsed)
}
