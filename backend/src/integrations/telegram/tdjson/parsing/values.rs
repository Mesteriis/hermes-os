use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;

pub(super) fn tdlib_string_id(value: &Value, field: &'static str) -> Result<String, TelegramError> {
    value
        .get(field)
        .map(tdlib_i64_value)
        .transpose()?
        .map(|value| value.to_string())
        .ok_or_else(|| TelegramError::TdlibRuntime(format!("TDLib field `{field}` is required")))
}

pub(super) fn tdlib_i64_value(value: &Value) -> Result<i64, TelegramError> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib id must be an i64".to_owned()))
}

pub(super) fn tdlib_unix_datetime_value(value: &Value) -> Result<DateTime<Utc>, TelegramError> {
    let timestamp = value
        .as_i64()
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib date must be an i64".to_owned()))?;
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .ok_or_else(|| TelegramError::TdlibRuntime(format!("invalid TDLib date `{timestamp}`")))
}
