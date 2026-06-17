use crate::integrations::telegram::client::TelegramError;
use crate::integrations::telegram::tdjson::{self, TdJsonClient, TelegramTdlibChatMemberSnapshot};

use super::super::TDJSON_COMMAND_TIMEOUT;
use super::responses::receive_tdlib_extra;

pub(super) fn actor_get_supergroup_members(
    client: &TdJsonClient,
    supergroup_id: i64,
    limit: i32,
) -> Result<Vec<TelegramTdlibChatMemberSnapshot>, TelegramError> {
    let extra = format!("hermes-supergroup-members-{supergroup_id}");
    client.send_json(&tdjson::tdlib_get_supergroup_members_request(
        supergroup_id,
        limit,
        &extra,
    ))?;
    let response = receive_tdlib_extra(client, &extra, TDJSON_COMMAND_TIMEOUT)?;
    if let Some(message) = tdjson::tdlib_error_message(&response) {
        return Err(TelegramError::TdlibRuntime(message));
    }
    tdjson::parse_tdlib_chat_member_list(&response)
}
