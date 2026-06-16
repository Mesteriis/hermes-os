use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};

pub(super) fn actor_edit_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    new_text: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = provider_message_id.trim().parse::<i64>().map_err(|_| {
        TelegramError::InvalidRequest("provider_message_id must be numeric".to_owned())
    })?;
    let extra = format!("hermes-edit-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_edit_message_text_request(
        chat_id, message_id, new_text, &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_delete_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    revoke: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = provider_message_id.trim().parse::<i64>().map_err(|_| {
        TelegramError::InvalidRequest("provider_message_id must be numeric".to_owned())
    })?;
    let extra = format!("hermes-delete-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_delete_messages_request(
        chat_id,
        &[message_id],
        revoke,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_set_reaction(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    reaction_emoji: &str,
    is_active: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = provider_message_id.trim().parse::<i64>().map_err(|_| {
        TelegramError::InvalidRequest("provider_message_id must be numeric".to_owned())
    })?;
    let extra = format!("hermes-reaction-{}", command_id.trim());
    let request = if is_active {
        tdjson::tdlib_add_message_reaction_request(chat_id, message_id, reaction_emoji, &extra)
    } else {
        tdjson::tdlib_remove_message_reaction_request(chat_id, message_id, reaction_emoji, &extra)
    };
    client.send_json(&request)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_pin_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    pin: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = provider_message_id.trim().parse::<i64>().map_err(|_| {
        TelegramError::InvalidRequest("provider_message_id must be numeric".to_owned())
    })?;
    let extra = format!("hermes-pin-{}", command_id.trim());
    let request = if pin {
        tdjson::tdlib_pin_chat_message_request(chat_id, message_id, false, &extra)
    } else {
        tdjson::tdlib_unpin_chat_message_request(chat_id, message_id, &extra)
    };
    client.send_json(&request)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}
