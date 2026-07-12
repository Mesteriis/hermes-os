use serde_json::json;

#[test]
fn parses_tdlib_file_snapshot_from_download_file_response() {
    let file = super::super::parse_tdlib_file_snapshot(&json!({
        "@type": "file",
        "id": 42,
        "size": 2048,
        "expected_size": 4096,
        "local": {
            "@type": "localFile",
            "path": "docker/data/telegram/account/files/document.pdf",
            "can_be_downloaded": true,
            "is_downloading_active": false,
            "is_downloading_completed": true,
            "downloaded_size": 2048
        },
        "remote": {
            "@type": "remoteFile",
            "id": "remote-file-id",
            "unique_id": "remote-unique-id",
            "is_uploading_active": false,
            "is_uploading_completed": false,
            "uploaded_size": 0
        }
    }))
    .expect("file snapshot");

    assert_eq!(file.file_id, 42);
    assert_eq!(file.size_bytes, Some(2048));
    assert_eq!(file.expected_size_bytes, Some(4096));
    assert_eq!(
        file.local_path.as_deref(),
        Some("docker/data/telegram/account/files/document.pdf")
    );
    assert!(file.is_downloading_completed);
    assert!(!file.is_downloading_active);
    assert_eq!(file.remote_unique_id.as_deref(), Some("remote-unique-id"));
    assert_eq!(file.downloaded_size_bytes, Some(2048));
}

#[test]
fn parses_tdlib_chat_snapshot_from_chat_object() {
    let chat = super::super::parse_tdlib_chat_snapshot(&json!({
        "@type": "chat",
        "id": 123456789,
        "type": {
            "@type": "chatTypeSupergroup",
            "supergroup_id": 555,
            "is_channel": true
        },
        "title": "Release Channel",
        "last_message": {
            "@type": "message",
            "id": 42,
            "date": 1781352000
        },
        "metadata": {"ignored": true}
    }))
    .expect("chat snapshot");

    assert_eq!(chat.provider_chat_id, "123456789");
    assert_eq!(chat.chat_kind.as_str(), "channel");
    assert_eq!(chat.title, "Release Channel");
    assert_eq!(chat.username, None);
    assert_eq!(
        chat.last_message_at.expect("last message").to_rfc3339(),
        "2026-06-13T12:00:00+00:00"
    );
    assert_eq!(chat.raw["@type"], "chat");
}

#[test]
fn parses_tdlib_chat_members_with_roles_and_permissions() {
    let members = super::super::parse_tdlib_chat_member_list(&json!({
        "@type": "chatMembers",
        "total_count": 2,
        "members": [
            {
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 42},
                "status": {
                    "@type": "chatMemberStatusCreator",
                    "is_member": true,
                    "custom_title": "Owner"
                }
            },
            {
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 43},
                "status": {
                    "@type": "chatMemberStatusAdministrator",
                    "can_be_edited": false,
                    "rights": {
                        "@type": "chatAdministratorRights",
                        "can_invite_users": true,
                        "can_delete_messages": true
                    }
                }
            }
        ]
    }))
    .expect("chat member list");

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].provider_member_id, "user:42");
    assert_eq!(members[0].role, "owner");
    assert!(members[0].is_owner);
    assert!(members[0].is_admin);
    assert_eq!(members[0].permissions["custom_title"], "Owner");
    assert_eq!(members[1].provider_member_id, "user:43");
    assert_eq!(members[1].role, "admin");
    assert!(members[1].is_admin);
    assert!(!members[1].is_owner);
    assert_eq!(members[1].permissions["rights"]["can_invite_users"], true);
}

#[test]
fn parses_tdlib_basic_group_full_info_members() {
    let members = super::super::parse_tdlib_basic_group_member_list(&json!({
        "@type": "basicGroupFullInfo",
        "creator_user_id": 42,
        "members": [
            {
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 42},
                "status": {
                    "@type": "chatMemberStatusCreator",
                    "is_member": true
                }
            },
            {
                "@type": "chatMember",
                "member_id": {"@type": "messageSenderUser", "user_id": 77},
                "status": {
                    "@type": "chatMemberStatusMember",
                    "member_until_date": 0
                }
            }
        ]
    }))
    .expect("basic group member list");

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].provider_member_id, "user:42");
    assert_eq!(members[0].role, "owner");
    assert_eq!(members[1].provider_member_id, "user:77");
    assert_eq!(members[1].role, "member");
    assert_eq!(members[1].status, "member");
}

#[test]
fn parses_tdlib_typing_update_from_user_chat_action() {
    let typing = super::super::parse_tdlib_typing_snapshot(&json!({
        "@type": "updateUserChatAction",
        "chat_id": -1001234567890_i64,
        "message_thread_id": 42,
        "sender_id": {
            "@type": "messageSenderUser",
            "user_id": 777
        },
        "action": {
            "@type": "chatActionTyping"
        }
    }))
    .expect("typing snapshot");

    assert_eq!(typing.provider_chat_id, "-1001234567890");
    assert_eq!(typing.provider_thread_id.as_deref(), Some("42"));
    assert_eq!(typing.sender_id, "user:777");
    assert_eq!(typing.action, "chatActionTyping");
    assert!(typing.is_active);
}

#[test]
fn parses_tdlib_typing_cancel_as_inactive() {
    let typing = super::super::parse_tdlib_typing_snapshot(&json!({
        "@type": "updateUserChatAction",
        "chat_id": -1001234567890_i64,
        "sender_id": {
            "@type": "messageSenderChat",
            "chat_id": -1009876543210_i64
        },
        "action": {
            "@type": "chatActionCancel"
        }
    }))
    .expect("typing snapshot");

    assert_eq!(typing.sender_id, "chat:-1009876543210");
    assert_eq!(typing.provider_thread_id, None);
    assert_eq!(typing.action, "chatActionCancel");
    assert!(!typing.is_active);
}

#[test]
fn parses_tdlib_topic_update_from_forum_topic_info() {
    let update = super::super::parse_tdlib_topic_update_snapshot(&json!({
        "@type": "updateForumTopicInfo",
        "chat_id": -1001234567890_i64,
        "info": {
            "@type": "forumTopicInfo",
            "message_thread_id": 42,
            "name": "Release notes",
            "icon": {
                "@type": "forumTopicIcon",
                "custom_emoji_id": "5368324170671202286"
            },
            "is_pinned": true,
            "is_closed": false
        }
    }))
    .expect("parse result")
    .expect("topic update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.topic.provider_topic_id, 42);
    assert_eq!(update.topic.title, "Release notes");
    assert_eq!(
        update.topic.icon_emoji.as_deref(),
        Some("5368324170671202286")
    );
    assert!(update.topic.is_pinned);
    assert!(!update.topic.is_closed);
    assert_eq!(update.topic.unread_count, 0);
    assert_eq!(update.topic.last_message_at, None);
}

#[test]
fn parses_tdlib_chat_read_inbox_update() {
    let update = super::super::parse_tdlib_chat_unread_snapshot(&json!({
        "@type": "updateChatReadInbox",
        "chat_id": -1001234567890_i64,
        "last_read_inbox_message_id": 777,
        "unread_count": 3
    }))
    .expect("parse result")
    .expect("unread update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.unread_count, Some(3));
    assert_eq!(update.unread_mention_count, None);
    assert_eq!(update.last_read_inbox_message_id.as_deref(), Some("777"));
    assert_eq!(update.source_event, "updateChatReadInbox");
}

#[test]
fn parses_tdlib_unread_mention_count_update() {
    let update = super::super::parse_tdlib_chat_unread_snapshot(&json!({
        "@type": "updateChatUnreadMentionCount",
        "chat_id": -1001234567890_i64,
        "unread_mention_count": 2
    }))
    .expect("parse result")
    .expect("unread mention update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.unread_count, None);
    assert_eq!(update.unread_mention_count, Some(2));
    assert_eq!(update.last_read_inbox_message_id, None);
    assert_eq!(update.source_event, "updateChatUnreadMentionCount");
}

#[test]
fn parses_tdlib_marked_as_unread_update() {
    let update = super::super::parse_tdlib_chat_marked_as_unread_snapshot(&json!({
        "@type": "updateChatIsMarkedAsUnread",
        "chat_id": -1001234567890_i64,
        "is_marked_as_unread": true
    }))
    .expect("parse result")
    .expect("marked unread update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert!(update.is_marked_as_unread);
    assert_eq!(update.source_event, "updateChatIsMarkedAsUnread");
}

#[test]
fn parses_tdlib_chat_notification_settings_update() {
    let update = super::super::parse_tdlib_chat_notification_settings_snapshot(&json!({
        "@type": "updateChatNotificationSettings",
        "chat_id": -1001234567890_i64,
        "notification_settings": {
            "@type": "chatNotificationSettings",
            "use_default_mute_for": false,
            "mute_for": 31708800
        }
    }))
    .expect("parse result")
    .expect("notification settings update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert!(!update.use_default_mute_for);
    assert_eq!(update.mute_for, 31_708_800);
    assert_eq!(update.source_event, "updateChatNotificationSettings");
}

#[test]
fn parses_tdlib_chat_position_update() {
    let update = super::super::parse_tdlib_chat_position_snapshot(&json!({
        "@type": "updateChatPosition",
        "chat_id": -1001234567890_i64,
        "position": {
            "@type": "chatPosition",
            "list": {
                "@type": "chatListArchive"
            },
            "order": 42,
            "is_pinned": false,
            "source": null
        }
    }))
    .expect("parse result")
    .expect("chat position update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.list_kind, "archive");
    assert_eq!(update.provider_folder_id, None);
    assert_eq!(update.order, 42);
    assert!(!update.is_pinned);
    assert_eq!(update.source_event, "updateChatPosition");
}

#[test]
fn ignores_tdlib_chat_position_with_unknown_list_type() {
    let update = super::super::parse_tdlib_chat_position_snapshot(&json!({
        "@type": "updateChatPosition",
        "chat_id": -1001234567890_i64,
        "position": {
            "@type": "chatPosition",
            "list": { "@type": "chatListFilter" },
            "order": 42,
            "is_pinned": false
        }
    }))
    .expect("parse result");

    assert!(update.is_none());
}

#[test]
fn ignores_incomplete_tdlib_chat_position_update() {
    let update = super::super::parse_tdlib_chat_position_snapshot(&json!({
        "@type": "updateChatPosition",
        "chat_id": -1001234567890_i64,
        "position": {
            "@type": "chatPosition",
            "list": { "@type": "chatListMain" }
        }
    }))
    .expect("parse result");

    assert!(update.is_none());
}

#[test]
fn parses_tdlib_chat_removed_from_list_snapshot() {
    let update = super::super::parse_tdlib_chat_removed_from_list_snapshot(&json!({
        "@type": "updateChatRemovedFromList",
        "chat_id": -1001234567890_i64,
        "chat_list": {
            "@type": "chatListFolder",
            "chat_folder_id": 7
        }
    }))
    .expect("parse result")
    .expect("chat removed from list update");

    assert_eq!(update.provider_chat_id, "-1001234567890");
    assert_eq!(update.list_kind, "folder");
    assert_eq!(update.provider_folder_id, Some(7));
    assert_eq!(update.source_event, "updateChatRemovedFromList");
    assert_eq!(update.raw["@type"], "updateChatRemovedFromList");
}

#[test]
fn parses_tdlib_chat_folder_snapshot() {
    let snapshot = super::super::parse_tdlib_chat_folder_snapshot(&json!({
        "@type": "chatFolder",
        "id": 7,
        "name": {
            "@type": "chatFolderName",
            "text": "Projects"
        },
        "icon": {
            "@type": "chatFolderIcon",
            "name": "Custom"
        },
        "color_id": 3
    }))
    .expect("parse result")
    .expect("chat folder snapshot");

    assert_eq!(snapshot.provider_folder_id, 7);
    assert_eq!(snapshot.title, "Projects");
    assert_eq!(snapshot.icon_name.as_deref(), Some("Custom"));
    assert_eq!(snapshot.color_id, Some(3));
}

#[test]
fn parses_tdlib_chat_folders_update() {
    let snapshot = super::super::parse_tdlib_chat_folders_update_snapshot(&json!({
        "@type": "updateChatFolders",
        "chat_folders": [{
            "@type": "chatFolder",
            "id": 7,
            "name": {
                "@type": "chatFolderName",
                "text": "Projects"
            },
            "icon": {
                "@type": "chatFolderIcon",
                "name": "Custom"
            }
        }]
    }))
    .expect("parse result")
    .expect("chat folders update");

    assert_eq!(snapshot.source_event, "updateChatFolders");
    assert_eq!(snapshot.folders.len(), 1);
    assert_eq!(snapshot.folders[0].provider_folder_id, 7);
    assert_eq!(snapshot.folders[0].title, "Projects");
}

#[test]
fn parses_tdlib_text_message_snapshot_from_message_object() {
    let message = super::super::parse_tdlib_message_snapshot(&json!({
        "@type": "message",
        "id": 777,
        "chat_id": 123456789,
        "sender_id": {
            "@type": "messageSenderUser",
            "user_id": 999
        },
        "date": 1781352060,
        "is_outgoing": false,
        "content": {
            "@type": "messageText",
            "text": {
                "@type": "formattedText",
                "text": "Incoming TDLib text",
                "entities": []
            }
        }
    }))
    .expect("message snapshot");

    assert_eq!(message.provider_chat_id, "123456789");
    assert_eq!(message.provider_message_id, "777");
    assert_eq!(message.sender_id, "user:999");
    assert_eq!(message.sender_display_name, "Telegram User 999");
    assert_eq!(message.text, "Incoming TDLib text");
    assert_eq!(message.delivery_state.as_str(), "received");
    assert_eq!(
        message.occurred_at.to_rfc3339(),
        "2026-06-13T12:01:00+00:00"
    );
    assert_eq!(message.raw["@type"], "message");
}

#[test]
fn parses_tdlib_media_message_without_caption_as_empty_text() {
    let message = super::super::parse_tdlib_message_snapshot(&json!({
        "@type": "message",
        "id": 778,
        "chat_id": 123456789,
        "sender_id": {
            "@type": "messageSenderUser",
            "user_id": 999
        },
        "date": 1781352061,
        "is_outgoing": false,
        "content": {
            "@type": "messagePhoto",
            "photo": {
                "@type": "photo",
                "sizes": []
            }
        }
    }))
    .expect("media message snapshot");

    assert_eq!(message.provider_message_id, "778");
    assert_eq!(message.text, "");
    assert_eq!(message.raw["content"]["@type"], "messagePhoto");
}

#[test]
fn recognizes_tdlib_bootstrap_error_events() {
    assert!(super::super::is_tdlib_parameters_not_specified_error(
        &json!({
            "@type": "error",
            "code": 400,
            "message": "Parameters aren't specified"
        })
    ));
    assert!(super::super::is_tdlib_database_encryption_key_needed_error(
        &json!({
            "@type": "error",
            "code": 400,
            "message": "Database encryption key is needed: call checkDatabaseEncryptionKey first"
        })
    ));
}

#[test]
fn parses_tdlib_user_identity_for_qr_account_defaults() {
    let identity = super::super::parse_tdlib_user_identity(&json!({
        "@type": "user",
        "id": 123456789,
        "usernames": {
            "active_usernames": ["Test_User"]
        }
    }))
    .expect("identity");

    assert_eq!(identity.user_id, "123456789");
    assert_eq!(identity.username.as_deref(), Some("Test_User"));
    assert_eq!(identity.suggested_account_id, "123456789_account_test_user");
    assert_eq!(identity.suggested_display_name, "@Test_User");
    assert_eq!(identity.suggested_external_account_id, "telegram:123456789");
}
