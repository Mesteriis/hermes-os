mod client;
mod identifiers;
mod library_paths;
mod parsing;
mod qr_login;
mod qr_login_support;
mod requests;
mod snapshots;

pub(crate) use self::client::{TdJsonClient, TdJsonLibrary, runtime_available};
pub(crate) use self::parsing::{
    authorization_state, is_tdlib_database_encryption_key_needed_error,
    is_tdlib_parameters_not_specified_error, parse_tdlib_chat_ids, parse_tdlib_chat_snapshot,
    parse_tdlib_file_snapshot, parse_tdlib_message_list, parse_tdlib_message_snapshot,
    parse_tdlib_topic_list, tdlib_error_message,
};
pub(crate) use self::qr_login::{cancel_qr_login, start_qr_login, submit_qr_login_password};
pub(crate) use self::qr_login_support::{PendingQrLoginMap, TelegramQrLoginSession};
pub(crate) use self::requests::{
    check_database_encryption_key_request, set_tdlib_parameters_request,
    tdlib_add_message_reaction_request, tdlib_database_directory, tdlib_delete_messages_request,
    tdlib_download_file_request, tdlib_edit_message_text_request, tdlib_get_chat_history_request,
    tdlib_get_chat_request, tdlib_get_chats_request, tdlib_get_forum_topics_request,
    tdlib_load_chats_request, tdlib_pin_chat_message_request,
    tdlib_remove_message_reaction_request, tdlib_send_reply_request,
    tdlib_send_text_message_request, tdlib_unpin_chat_message_request,
};
pub(crate) use self::snapshots::{
    TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot, TelegramTdlibMessageSnapshot,
    TelegramTdlibTopicSnapshot,
};

#[cfg(test)]
use self::library_paths::{tdjson_library_candidates_with_context, tdjson_platform_dir};
#[cfg(test)]
use self::qr_login::cancel_existing_qr_logins_for_account;
#[cfg(test)]
use self::qr_login_support::{
    TelegramQrLoginCommand, TelegramQrLoginIdentity, mark_worker_complete, new_worker_completion,
    parse_tdlib_user_identity, password_waiting_response, qr_preparing_response, ready_response,
    render_qr_svg, state_allows_qr_request,
};

#[cfg(test)]
mod tests;
