use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibFileSnapshot};
use hermes_provider_telegram::tdlib::chats::download_file;

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::receive_tdlib_extra;

pub(super) fn actor_download_file(
    client: &TdJsonClient,
    file_id: i64,
    priority: i32,
) -> Result<TelegramTdlibFileSnapshot, TelegramError> {
    let extra = format!("hermes-runtime-download-file-{file_id}");
    client.send_json(&download_file(file_id, priority, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_file_snapshot(&response)
}
