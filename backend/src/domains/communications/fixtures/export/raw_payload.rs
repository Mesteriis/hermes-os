use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde_json::Value;

use super::errors::EmailFixtureExportError;

pub(super) fn raw_rfc822_bytes(payload: &Value) -> Result<Vec<u8>, EmailFixtureExportError> {
    let raw = payload
        .get("raw_rfc822_base64")
        .and_then(Value::as_str)
        .ok_or(EmailFixtureExportError::MissingRawRfc822)?;
    BASE64_STANDARD
        .decode(raw)
        .map_err(EmailFixtureExportError::InvalidRawBase64)
}
