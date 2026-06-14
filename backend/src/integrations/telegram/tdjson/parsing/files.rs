use serde_json::Value;

use crate::integrations::telegram::client::TelegramError;

use super::values::tdlib_i64_value;
use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibFileSnapshot;

pub(crate) fn parse_tdlib_file_snapshot(
    file: &Value,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    if file.get("@type").and_then(Value::as_str) != Some("file") {
        return Err(TelegramError::TdlibRuntime(
            "TDLib file snapshot must have @type=file".to_owned(),
        ));
    }

    let file_id = file
        .get("id")
        .map(tdlib_i64_value)
        .transpose()?
        .ok_or_else(|| TelegramError::TdlibRuntime("TDLib file id is required".to_owned()))?;
    let local = file.get("local").and_then(Value::as_object);
    let remote = file.get("remote").and_then(Value::as_object);
    let local_path = local
        .and_then(|value| value.get("path"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let is_downloading_active = local
        .and_then(|value| value.get("is_downloading_active"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let is_downloading_completed = local
        .and_then(|value| value.get("is_downloading_completed"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let downloaded_size_bytes = local
        .and_then(|value| value.get("downloaded_size"))
        .map(tdlib_i64_value)
        .transpose()?;
    let remote_id = remote
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let remote_unique_id = remote
        .and_then(|value| value.get("unique_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Ok(TelegramTdlibFileSnapshot {
        file_id,
        size_bytes: file.get("size").map(tdlib_i64_value).transpose()?,
        expected_size_bytes: file.get("expected_size").map(tdlib_i64_value).transpose()?,
        local_path,
        is_downloading_active,
        is_downloading_completed,
        downloaded_size_bytes,
        remote_id,
        remote_unique_id,
        raw: file.clone(),
    })
}
