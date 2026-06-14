use std::time::{Duration, Instant};

use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient};

use super::super::TDJSON_RECEIVE_POLL_SECONDS;

pub(super) fn receive_tdlib_extra(
    client: &TdJsonClient,
    expected_extra: &str,
    timeout: Duration,
) -> Result<Value, TelegramError> {
    let started_at = Instant::now();
    while started_at.elapsed() < timeout {
        let Some(event) = client.receive_json(TDJSON_RECEIVE_POLL_SECONDS)? else {
            continue;
        };
        if event.get("@extra").and_then(Value::as_str) == Some(expected_extra) {
            return Ok(event);
        }
        if let Some(message) = tdjson::tdlib_error_message(&event) {
            tracing::debug!(error = %message, "ignored unrelated TDLib error while waiting for correlated response");
        }
    }
    Err(TelegramError::TdlibRuntime(format!(
        "TDLib request `{expected_extra}` timed out"
    )))
}

pub(super) fn tdlib_provider_chat_id(provider_chat_id: &str) -> Result<i64, TelegramError> {
    provider_chat_id.trim().parse::<i64>().map_err(|_| {
        TelegramError::InvalidRequest(format!(
            "TDLib provider_chat_id `{}` must be a Telegram numeric chat id",
            provider_chat_id.trim()
        ))
    })
}
