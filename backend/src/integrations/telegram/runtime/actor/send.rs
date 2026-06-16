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

pub(super) fn actor_send_reply(
    client: &TdJsonClient,
    provider_chat_id: &str,
    reply_to_provider_message_id: &str,
    text: &str,
    command_id: &str,
) -> Result<TelegramTdlibMessageSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let reply_to_message_id = reply_to_provider_message_id
        .trim()
        .parse::<i64>()
        .map_err(|_| {
            TelegramError::InvalidRequest("reply_to_provider_message_id must be numeric".to_owned())
        })?;
    let extra = format!("hermes-reply-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_send_reply_request(
        chat_id,
        reply_to_message_id,
        text,
        &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_message_snapshot(&response)
}
