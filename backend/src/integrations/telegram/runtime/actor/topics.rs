use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibTopicSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::{receive_tdlib_extra, tdlib_provider_chat_id};

pub(super) fn actor_get_forum_topics(
    client: &TdJsonClient,
    provider_chat_id: &str,
    limit: i32,
) -> Result<Vec<TelegramTdlibTopicSnapshot>, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("hermes-forum-topics-{provider_chat_id}");
    client.send_json(&tdjson::tdlib_get_forum_topics_request(
        chat_id, limit, &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_topic_list(&response)
}

pub(super) fn actor_create_forum_topic(
    client: &TdJsonClient,
    provider_chat_id: &str,
    title: &str,
    command_id: &str,
) -> Result<TelegramTdlibTopicSnapshot, TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("hermes-forum-topic-create-{command_id}");
    client.send_json(&tdjson::tdlib_create_forum_topic_request(
        chat_id, title, &extra,
    )?)?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_created_forum_topic(&response)
}

pub(super) fn actor_toggle_forum_topic_closed(
    client: &TdJsonClient,
    provider_chat_id: &str,
    provider_topic_id: i64,
    is_closed: bool,
    command_id: &str,
) -> Result<(), TelegramError> {
    let chat_id = tdlib_provider_chat_id(provider_chat_id)?;
    let extra = format!("hermes-forum-topic-closed-{command_id}");
    client.send_json(&tdjson::tdlib_toggle_forum_topic_is_closed_request(
        chat_id,
        provider_topic_id,
        is_closed,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    Ok(())
}
