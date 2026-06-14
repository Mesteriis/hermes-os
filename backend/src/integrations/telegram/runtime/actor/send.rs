use crate::integrations::telegram::client::{TelegramError, TelegramManualSendRequest};
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibMessageSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};

pub(super) fn actor_send_text(
    client: &TdJsonClient,
    request: &TelegramManualSendRequest,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(&request.provider_chat_id)?;
    let extra = format!("hermes-runtime-send-{}", request.command_id.trim());
    client.send_json(&tdjson::tdlib_send_text_message_request(
        chat_id,
        &request.text,
        &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_snapshot(&response)
}
