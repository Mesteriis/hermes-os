mod client;
mod folder_requests;
mod identifiers;
mod library_paths;
mod parsing;
mod qr_login;
mod qr_login_support;
mod requests;
mod snapshots;

pub(crate) use self::client::{TdJsonClient, TdJsonLibrary, runtime_available};
pub(crate) use self::folder_requests::tdlib_edit_chat_folder_remove_chat_request;
pub(crate) use self::parsing::{
    TelegramTdlibChatMarkedAsUnreadSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatRemovedFromListSnapshot,
    TelegramTdlibChatUnreadSnapshot, TelegramTdlibTopicUpdateSnapshot, TelegramTdlibTypingSnapshot,
    authorization_state, is_tdlib_database_encryption_key_needed_error,
    is_tdlib_parameters_not_specified_error, parse_tdlib_basic_group_member_list,
    parse_tdlib_chat_folder_snapshot, parse_tdlib_chat_folders_update_snapshot,
    parse_tdlib_chat_ids, parse_tdlib_chat_marked_as_unread_snapshot, parse_tdlib_chat_member_list,
    parse_tdlib_chat_notification_settings_snapshot, parse_tdlib_chat_position_snapshot,
    parse_tdlib_chat_removed_from_list_snapshot, parse_tdlib_chat_snapshot,
    parse_tdlib_chat_unread_snapshot, parse_tdlib_created_forum_topic, parse_tdlib_file_snapshot,
    parse_tdlib_message_content_snapshot, parse_tdlib_message_delete_snapshot,
    parse_tdlib_message_edited_snapshot, parse_tdlib_message_interaction_info_snapshot,
    parse_tdlib_message_list, parse_tdlib_message_pinned_snapshot, parse_tdlib_message_snapshot,
    parse_tdlib_new_message_snapshot, parse_tdlib_topic_list, parse_tdlib_topic_update_snapshot,
    parse_tdlib_typing_snapshot, tdlib_error_message,
};
pub(crate) use self::qr_login::{cancel_qr_login, start_qr_login, submit_qr_login_password};
pub(crate) use self::qr_login_support::{PendingQrLoginMap, TelegramQrLoginSession};
pub(crate) use self::requests::{
    check_database_encryption_key_request, set_tdlib_parameters_request,
    tdlib_add_chat_to_folder_request, tdlib_add_chat_to_list_request,
    tdlib_add_message_reaction_request, tdlib_create_forum_topic_request, tdlib_database_directory,
    tdlib_delete_messages_request, tdlib_download_file_request, tdlib_edit_message_text_request,
    tdlib_get_basic_group_full_info_request, tdlib_get_basic_group_request,
    tdlib_get_chat_folder_request, tdlib_get_chat_history_request, tdlib_get_chat_request,
    tdlib_get_chats_request, tdlib_get_forum_topics_request,
    tdlib_get_supergroup_administrators_request, tdlib_get_supergroup_members_request,
    tdlib_join_chat_request, tdlib_leave_chat_request, tdlib_load_chats_request,
    tdlib_pin_chat_message_request, tdlib_remove_message_reaction_request,
    tdlib_search_chat_messages_request, tdlib_search_messages_request, tdlib_send_forward_request,
    tdlib_send_media_message_request, tdlib_send_reply_request, tdlib_send_text_message_request,
    tdlib_set_chat_mute_request, tdlib_toggle_chat_marked_as_unread_request,
    tdlib_toggle_forum_topic_is_closed_request, tdlib_unpin_chat_message_request,
    tdlib_view_messages_request,
};
pub(crate) use self::snapshots::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatMemberSnapshot, TelegramTdlibChatSnapshot,
    TelegramTdlibFileSnapshot, TelegramTdlibMessageContentSnapshot,
    TelegramTdlibMessageDeleteSnapshot, TelegramTdlibMessageEditedSnapshot,
    TelegramTdlibMessageInteractionInfoSnapshot, TelegramTdlibMessagePinnedSnapshot,
    TelegramTdlibMessageSnapshot, TelegramTdlibTopicSnapshot,
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
