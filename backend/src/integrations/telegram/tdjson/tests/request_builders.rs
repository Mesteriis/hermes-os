use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;

use crate::integrations::telegram::client::TelegramQrLoginStartRequest;
use crate::integrations::telegram::runtime::TelegramMediaSendType;

#[test]
fn tdlib_parameters_use_legacy_nested_shape_for_tdlib_1_8_runtime() {
    let request = TelegramQrLoginStartRequest {
        account_id: "telegram-qr".to_owned(),
        display_name: "Telegram QR".to_owned(),
        external_account_id: "qr-login:telegram-qr".to_owned(),
        api_id: Some(12345),
        api_hash: Some("telegram-api-hash".to_owned()),
        session_encryption_key: Some("telegram-session-key".to_owned()),
        tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
        transcription_enabled: true,
    };

    let command = super::super::set_tdlib_parameters_request(
        &request,
        Path::new("docker/data/telegram/telegram-qr"),
    )
    .expect("TDLib parameters");

    assert_eq!(command["@type"], "setTdlibParameters");
    assert_eq!(command["parameters"]["api_id"], 12345);
    assert_eq!(command["parameters"]["api_hash"], "telegram-api-hash");
    assert_eq!(command["parameters"]["enable_storage_optimizer"], true);
    assert_eq!(command["parameters"]["ignore_file_names"], false);
    assert_eq!(
        command["parameters"]["database_encryption_key"],
        STANDARD.encode("telegram-session-key")
    );
    assert_eq!(command["database_encryption_key"], serde_json::Value::Null);
}

#[test]
fn tdlib_database_key_check_uses_same_base64_key_without_plaintext_secret() {
    let request = TelegramQrLoginStartRequest {
        account_id: "telegram-qr".to_owned(),
        display_name: "Telegram QR".to_owned(),
        external_account_id: "qr-login:telegram-qr".to_owned(),
        api_id: Some(12345),
        api_hash: Some("telegram-api-hash".to_owned()),
        session_encryption_key: Some("telegram-session-key".to_owned()),
        tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
        transcription_enabled: true,
    };

    let command = super::super::check_database_encryption_key_request(&request);

    assert_eq!(command["@type"], "checkDatabaseEncryptionKey");
    assert_eq!(
        command["encryption_key"],
        STANDARD.encode("telegram-session-key")
    );
    assert_ne!(command["encryption_key"], "telegram-session-key");
}

#[test]
fn tdlib_send_text_message_request_uses_formatted_text_content() {
    let command = super::super::tdlib_send_text_message_request(
        123456789,
        "Hello from Hermes",
        "hermes-send-message-1",
    )
    .expect("send message request");

    assert_eq!(command["@type"], "sendMessage");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["@extra"], "hermes-send-message-1");
    assert_eq!(
        command["input_message_content"]["@type"],
        "inputMessageText"
    );
    assert_eq!(
        command["input_message_content"]["text"]["@type"],
        "formattedText"
    );
    assert_eq!(
        command["input_message_content"]["text"]["text"],
        "Hello from Hermes"
    );
    assert_eq!(
        command["input_message_content"]["text"]["entities"],
        json!([])
    );
    assert_eq!(command["input_message_content"]["clear_draft"], true);
}

#[test]
fn tdlib_send_media_message_request_uses_local_document_content() {
    let command = super::super::tdlib_send_media_message_request(
        123456789,
        TelegramMediaSendType::Document,
        "/tmp/hermes/upload.pdf",
        Some("Document caption"),
        Some("upload.pdf"),
        "hermes-send-media-1",
    )
    .expect("send media request");

    assert_eq!(command["@type"], "sendMessage");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["@extra"], "hermes-send-media-1");
    assert_eq!(
        command["input_message_content"]["@type"],
        "inputMessageDocument"
    );
    assert_eq!(
        command["input_message_content"]["document"]["@type"],
        "inputFileLocal"
    );
    assert_eq!(
        command["input_message_content"]["document"]["path"],
        "/tmp/hermes/upload.pdf"
    );
    assert_eq!(
        command["input_message_content"]["caption"]["text"],
        "Document caption"
    );
}

#[test]
fn tdlib_send_media_message_request_rejects_empty_local_path() {
    let result = super::super::tdlib_send_media_message_request(
        123456789,
        TelegramMediaSendType::Photo,
        "   ",
        None,
        None,
        "hermes-send-media-empty",
    );

    assert!(result.is_err());
}

#[test]
fn tdlib_get_chat_history_request_caps_limit_to_tdlib_page_size() {
    let command = super::super::tdlib_get_chat_history_request(
        123456789,
        Some(98765),
        500,
        true,
        "hermes-history-1",
    );

    assert_eq!(command["@type"], "getChatHistory");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["from_message_id"], 98765);
    assert_eq!(command["offset"], 0);
    assert_eq!(command["limit"], 100);
    assert_eq!(command["only_local"], true);
    assert_eq!(command["@extra"], "hermes-history-1");
}

#[test]
fn tdlib_download_file_request_uses_synchronous_on_demand_download() {
    let command = super::super::tdlib_download_file_request(42, 16, "hermes-download-file-42");

    assert_eq!(command["@type"], "downloadFile");
    assert_eq!(command["file_id"], 42);
    assert_eq!(command["priority"], 16);
    assert_eq!(command["offset"], 0);
    assert_eq!(command["limit"], 0);
    assert_eq!(command["synchronous"], true);
    assert_eq!(command["@extra"], "hermes-download-file-42");
}

#[test]
fn tdlib_create_forum_topic_request_uses_expected_shape() {
    let command = super::super::tdlib_create_forum_topic_request(
        123456789,
        "Release planning",
        "hermes-topic-create-1",
    )
    .expect("topic create request");

    assert_eq!(command["@type"], "createForumTopic");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["name"], "Release planning");
    assert_eq!(command["icon_custom_emoji_id"], 0);
    assert_eq!(command["@extra"], "hermes-topic-create-1");
}

#[test]
fn tdlib_create_forum_topic_request_rejects_empty_title() {
    let result =
        super::super::tdlib_create_forum_topic_request(123456789, "   ", "hermes-topic-create-2");

    assert!(result.is_err());
}

#[test]
fn tdlib_edit_chat_folder_remove_chat_request_preserves_shape_and_excludes_chat() {
    let command = super::super::tdlib_edit_chat_folder_remove_chat_request(
        7,
        222,
        &json!({
            "@type": "chatFolder",
            "name": {
                "@type": "chatFolderName",
                "text": "Projects",
                "animate_custom_emoji": false
            },
            "icon": {
                "@type": "chatFolderIcon",
                "name": "Custom"
            },
            "color_id": 3,
            "is_shareable": false,
            "pinned_chat_ids": [111, 222],
            "included_chat_ids": [222, 333],
            "excluded_chat_ids": [444],
            "exclude_muted": false,
            "exclude_read": true,
            "exclude_archived": false,
            "include_contacts": true,
            "include_non_contacts": false,
            "include_bots": false,
            "include_groups": true,
            "include_channels": true
        }),
        "hermes-folder-remove-1",
    )
    .expect("folder remove request");

    assert_eq!(command["@type"], "editChatFolder");
    assert_eq!(command["chat_folder_id"], 7);
    assert_eq!(command["@extra"], "hermes-folder-remove-1");
    assert_eq!(command["folder"]["name"]["text"], "Projects");
    assert_eq!(command["folder"]["icon"]["name"], "Custom");
    assert_eq!(command["folder"]["pinned_chat_ids"], json!([111]));
    assert_eq!(command["folder"]["included_chat_ids"], json!([333]));
    assert_eq!(command["folder"]["excluded_chat_ids"], json!([444, 222]));
    assert_eq!(command["folder"]["exclude_read"], true);
    assert_eq!(command["folder"]["include_channels"], true);
}

#[test]
fn tdlib_toggle_forum_topic_is_closed_request_uses_expected_shape() {
    let command = super::super::tdlib_toggle_forum_topic_is_closed_request(
        123456789,
        555,
        true,
        "hermes-topic-close-1",
    );

    assert_eq!(command["@type"], "toggleForumTopicIsClosed");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_thread_id"], 555);
    assert_eq!(command["is_closed"], true);
    assert_eq!(command["@extra"], "hermes-topic-close-1");
}

#[test]
fn tdlib_edit_message_text_request_uses_edit_message_text_type() {
    let command = super::super::tdlib_edit_message_text_request(
        123456789,
        987654321,
        "Updated text",
        "hermes-edit-cmd-1",
    )
    .expect("edit message request");

    assert_eq!(command["@type"], "editMessageText");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_id"], 987654321);
    assert_eq!(command["@extra"], "hermes-edit-cmd-1");
    assert_eq!(
        command["input_message_content"]["@type"],
        "inputMessageText"
    );
    assert_eq!(
        command["input_message_content"]["text"]["text"],
        "Updated text"
    );
}

#[test]
fn tdlib_edit_message_text_request_rejects_empty_text() {
    let result = super::super::tdlib_edit_message_text_request(123, 456, "   ", "hermes-edit-1");
    assert!(result.is_err());
}

#[test]
fn tdlib_delete_messages_request_uses_delete_messages_type() {
    let command = super::super::tdlib_delete_messages_request(
        123456789,
        &[111, 222],
        true,
        "hermes-delete-1",
    );

    assert_eq!(command["@type"], "deleteMessages");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_ids"], json!([111, 222]));
    assert_eq!(command["revoke"], true);
    assert_eq!(command["@extra"], "hermes-delete-1");
}

#[test]
fn tdlib_add_message_reaction_request_uses_add_message_reaction_type() {
    let command = super::super::tdlib_add_message_reaction_request(
        123456789,
        987654321,
        "👍",
        "hermes-react-1",
    );

    assert_eq!(command["@type"], "addMessageReaction");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_id"], 987654321);
    assert_eq!(command["reaction_type"]["@type"], "reactionTypeEmoji");
    assert_eq!(command["reaction_type"]["emoji"], "👍");
    assert_eq!(command["is_big"], false);
    assert_eq!(command["@extra"], "hermes-react-1");
}

#[test]
fn tdlib_remove_message_reaction_request_uses_remove_message_reaction_type() {
    let command = super::super::tdlib_remove_message_reaction_request(
        123456789,
        987654321,
        "👍",
        "hermes-unreact-1",
    );

    assert_eq!(command["@type"], "removeMessageReaction");
    assert_eq!(command["reaction_type"]["emoji"], "👍");
    assert_eq!(command["@extra"], "hermes-unreact-1");
}

#[test]
fn tdlib_pin_chat_message_request_uses_pin_chat_message_type() {
    let command =
        super::super::tdlib_pin_chat_message_request(123456789, 987654321, false, "hermes-pin-1");

    assert_eq!(command["@type"], "pinChatMessage");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_id"], 987654321);
    assert_eq!(command["disable_notification"], false);
    assert_eq!(command["only_for_self"], false);
    assert_eq!(command["@extra"], "hermes-pin-1");
}

#[test]
fn tdlib_unpin_chat_message_request_uses_unpin_chat_message_type() {
    let command =
        super::super::tdlib_unpin_chat_message_request(123456789, 987654321, "hermes-unpin-1");

    assert_eq!(command["@type"], "unpinChatMessage");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_id"], 987654321);
    assert_eq!(command["@extra"], "hermes-unpin-1");
}

#[test]
fn tdlib_toggle_chat_marked_as_unread_request_uses_toggle_type() {
    let command = super::super::tdlib_toggle_chat_marked_as_unread_request(
        123456789,
        true,
        "hermes-unread-1",
    );

    assert_eq!(command["@type"], "toggleChatIsMarkedAsUnread");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["is_marked_as_unread"], true);
    assert_eq!(command["@extra"], "hermes-unread-1");
}

#[test]
fn tdlib_view_messages_request_uses_force_read_view_messages_type() {
    let command =
        super::super::tdlib_view_messages_request(123456789, &[987654321], true, "hermes-read-1");

    assert_eq!(command["@type"], "viewMessages");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["message_ids"], json!([987654321]));
    assert_eq!(command["source"], serde_json::Value::Null);
    assert_eq!(command["force_read"], true);
    assert_eq!(command["@extra"], "hermes-read-1");
}

#[test]
fn tdlib_send_forward_request_uses_forward_messages_type() {
    let command = super::super::tdlib_send_forward_request(
        123456789,
        987654321,
        111222333,
        "hermes-forward-1",
    );

    assert_eq!(command["@type"], "forwardMessages");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["from_chat_id"], 987654321);
    assert_eq!(command["message_ids"], json!([111222333]));
    assert_eq!(command["send_copy"], false);
    assert_eq!(command["remove_caption"], false);
    assert_eq!(command["@extra"], "hermes-forward-1");
}

#[test]
fn tdlib_add_chat_to_list_request_uses_archive_chat_list() {
    let command = super::super::tdlib_add_chat_to_list_request(123456789, true, "hermes-archive-1");

    assert_eq!(command["@type"], "addChatToList");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["chat_list"]["@type"], "chatListArchive");
    assert_eq!(command["@extra"], "hermes-archive-1");
}

#[test]
fn tdlib_add_chat_to_list_request_uses_main_chat_list() {
    let command =
        super::super::tdlib_add_chat_to_list_request(123456789, false, "hermes-unarchive-1");

    assert_eq!(command["@type"], "addChatToList");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["chat_list"]["@type"], "chatListMain");
    assert_eq!(command["@extra"], "hermes-unarchive-1");
}

#[test]
fn tdlib_get_chat_folder_request_uses_get_chat_folder_type() {
    let command = super::super::tdlib_get_chat_folder_request(7, "hermes-folder-7");

    assert_eq!(command["@type"], "getChatFolder");
    assert_eq!(command["chat_folder_id"], 7);
    assert_eq!(command["@extra"], "hermes-folder-7");
}

#[test]
fn tdlib_set_chat_mute_request_uses_notification_settings() {
    let command = super::super::tdlib_set_chat_mute_request(123456789, true, "hermes-mute-1");

    assert_eq!(command["@type"], "setChatNotificationSettings");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(
        command["notification_settings"]["@type"],
        "chatNotificationSettings"
    );
    assert_eq!(
        command["notification_settings"]["use_default_mute_for"],
        false
    );
    assert_eq!(command["notification_settings"]["mute_for"], 31_708_800);
    assert_eq!(command["@extra"], "hermes-mute-1");
}

#[test]
fn tdlib_set_chat_mute_request_uses_default_mute_for_unmute() {
    let command = super::super::tdlib_set_chat_mute_request(123456789, false, "hermes-unmute-1");

    assert_eq!(command["@type"], "setChatNotificationSettings");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(
        command["notification_settings"]["use_default_mute_for"],
        true
    );
    assert_eq!(command["notification_settings"]["mute_for"], 0);
    assert_eq!(command["@extra"], "hermes-unmute-1");
}

#[test]
fn tdlib_get_supergroup_members_request_uses_recent_filter() {
    let command =
        super::super::tdlib_get_supergroup_members_request(555, 25, 250, "hermes-members-555");

    assert_eq!(command["@type"], "getSupergroupMembers");
    assert_eq!(command["supergroup_id"], 555);
    assert_eq!(command["filter"]["@type"], "supergroupMembersFilterRecent");
    assert_eq!(command["offset"], 25);
    assert_eq!(command["limit"], 100);
    assert_eq!(command["@extra"], "hermes-members-555");
}

#[test]
fn tdlib_get_supergroup_administrators_request_uses_admin_filter() {
    let command = super::super::tdlib_get_supergroup_administrators_request(
        555,
        10,
        250,
        "hermes-members-admins-555",
    );

    assert_eq!(command["@type"], "getSupergroupMembers");
    assert_eq!(command["supergroup_id"], 555);
    assert_eq!(
        command["filter"]["@type"],
        "supergroupMembersFilterAdministrators"
    );
    assert_eq!(command["offset"], 10);
    assert_eq!(command["limit"], 100);
    assert_eq!(command["@extra"], "hermes-members-admins-555");
}

#[test]
fn tdlib_get_basic_group_request_uses_basic_group_type() {
    let command = super::super::tdlib_get_basic_group_request(321, "hermes-basic-group-321");

    assert_eq!(command["@type"], "getBasicGroup");
    assert_eq!(command["basic_group_id"], 321);
    assert_eq!(command["@extra"], "hermes-basic-group-321");
}

#[test]
fn tdlib_get_basic_group_full_info_request_uses_expected_shape() {
    let command = super::super::tdlib_get_basic_group_full_info_request(
        321,
        "hermes-basic-group-full-info-321",
    );

    assert_eq!(command["@type"], "getBasicGroupFullInfo");
    assert_eq!(command["basic_group_id"], 321);
    assert_eq!(command["@extra"], "hermes-basic-group-full-info-321");
}

#[test]
fn tdlib_join_chat_request_uses_join_chat_type() {
    let command = super::super::tdlib_join_chat_request(123456789, "hermes-join-1");

    assert_eq!(command["@type"], "joinChat");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["@extra"], "hermes-join-1");
}

#[test]
fn tdlib_leave_chat_request_uses_leave_chat_type() {
    let command = super::super::tdlib_leave_chat_request(123456789, "hermes-leave-1");

    assert_eq!(command["@type"], "leaveChat");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["@extra"], "hermes-leave-1");
}

#[test]
fn tdlib_add_chat_to_folder_request_uses_chat_list_folder() {
    let command = super::super::tdlib_add_chat_to_folder_request(123456789, 7, " folder-extra ");

    assert_eq!(command["@type"], "addChatToList");
    assert_eq!(command["chat_id"], 123456789);
    assert_eq!(command["chat_list"]["@type"], "chatListFolder");
    assert_eq!(command["chat_list"]["chat_folder_id"], 7);
    assert_eq!(command["@extra"], "folder-extra");
}
