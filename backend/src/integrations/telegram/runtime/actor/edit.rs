use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient};
use hermes_provider_telegram::tdlib::edit_message_text;

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id, tdlib_provider_message_id};

pub(super) fn actor_edit_message(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_message_id: &str,
    new_text: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let message_id = tdlib_provider_message_id(provider_message_id)?;
    let extra = format!("hermes-edit-{}", command_id.trim());
    client.send_json(
        &edit_message_text(chat_id, message_id, new_text, &extra)
            .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?,
    )?;
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
    let message_id = tdlib_provider_message_id(provider_message_id)?;
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
    let message_id = tdlib_provider_message_id(provider_message_id)?;
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
    let message_id = tdlib_provider_message_id(provider_message_id)?;
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

pub(super) fn actor_toggle_chat_unread(
    client: &TdJsonClient,
    provider_chat_id: &str,
    is_marked_as_unread: bool,
    read_through_provider_message_id: Option<&str>,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    if is_marked_as_unread {
        let extra = format!("hermes-chat-unread-{}", command_id.trim());
        client.send_json(&tdjson::tdlib_toggle_chat_marked_as_unread_request(
            chat_id, true, &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        return Ok(());
    }

    if let Some(provider_message_id) = read_through_provider_message_id {
        let message_id = tdlib_provider_message_id(provider_message_id)?;
        let extra = format!("hermes-chat-read-{}", command_id.trim());
        client.send_json(&tdjson::tdlib_view_messages_request(
            chat_id,
            &[message_id],
            true,
            &extra,
        ))?;
        let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
        if let Some(message) = tdjson::tdlib_error_message(&response) {
            return Err(TelegramError::TdlibRuntime(message));
        }
        return Ok(());
    }

    let extra = format!("hermes-chat-read-toggle-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_toggle_chat_marked_as_unread_request(
        chat_id, false, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_toggle_chat_archive(
    client: &TdJsonClient,
    provider_chat_id: &str,
    archived: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("hermes-chat-archive-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_add_chat_to_list_request(
        chat_id, archived, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_toggle_chat_mute(
    client: &TdJsonClient,
    provider_chat_id: &str,
    muted: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("hermes-chat-mute-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_set_chat_mute_request(chat_id, muted, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_add_chat_to_folder(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_folder_id: i64,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("hermes-chat-folder-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_add_chat_to_folder_request(
        chat_id,
        provider_folder_id,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_remove_chat_from_folder(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_folder_id: i64,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let get_extra = format!("hermes-chat-folder-remove-get-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_get_chat_folder_request(
        provider_folder_id,
        &get_extra,
    ))?;
    let folder_response = receive_tdlib_extra(client, &get_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&folder_response) {
        return Err(TelegramError::TdlibRuntime(message));
    }

    let edit_extra = format!("hermes-chat-folder-remove-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_edit_chat_folder_remove_chat_request(
        provider_folder_id,
        chat_id,
        &folder_response,
        &edit_extra,
    )?)?;
    let response = receive_tdlib_extra(client, &edit_extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_join_chat(
    client: &TdJsonClient,
    provider_chat_id: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("hermes-chat-join-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_join_chat_request(chat_id, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}

pub(super) fn actor_leave_chat(
    client: &TdJsonClient,
    provider_chat_id: &str,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("hermes-chat-leave-{}", command_id.trim());
    client.send_json(&tdjson::tdlib_leave_chat_request(chat_id, &extra))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}
