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
