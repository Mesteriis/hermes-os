use serde_json::json;

use super::{
    TelegramChatMember, inactive_roster_membership_state, tdlib_self_membership_lifecycle,
    telegram_self_provider_member_id,
};

#[test]
fn derives_self_provider_member_id_only_from_numeric_telegram_identity() {
    assert_eq!(
        telegram_self_provider_member_id("telegram:12345").as_deref(),
        Some("user:12345")
    );
    assert_eq!(
        telegram_self_provider_member_id("user:456").as_deref(),
        Some("user:456")
    );
    assert_eq!(telegram_self_provider_member_id("fixture-user"), None);
    assert_eq!(
        telegram_self_provider_member_id("telegram:not-numeric"),
        None
    );
}

#[test]
fn parses_self_leave_and_join_membership_evidence_from_tdlib_service_messages() {
    let leave = tdlib_self_membership_lifecycle(
        "telegram:42",
        &json!({
            "content": {
                "@type": "messageChatDeleteMember",
                "user_id": 42
            }
        }),
    )
    .expect("leave lifecycle");
    assert_eq!(leave.command_kind, "leave");
    assert_eq!(leave.provider_member_id, "user:42");
    assert_eq!(leave.observed_via, "tdlib.messageChatDeleteMember");
    assert_eq!(leave.membership_state, "left");

    let join = tdlib_self_membership_lifecycle(
        "telegram:42",
        &json!({
            "content": {
                "@type": "messageChatAddMembers",
                "member_user_ids": [1, 42, 9]
            }
        }),
    )
    .expect("join lifecycle");
    assert_eq!(join.command_kind, "join");
    assert_eq!(join.provider_member_id, "user:42");
    assert_eq!(join.observed_via, "tdlib.messageChatAddMembers");
    assert_eq!(join.membership_state, "present");
}

#[test]
fn ignores_service_messages_for_other_members_or_unsupported_content() {
    assert!(
        tdlib_self_membership_lifecycle(
            "telegram:42",
            &json!({
                "content": {
                    "@type": "messageChatDeleteMember",
                    "user_id": 7
                }
            }),
        )
        .is_none()
    );
    assert!(
        tdlib_self_membership_lifecycle(
            "telegram:42",
            &json!({
                "content": {
                    "@type": "messageChatJoinByLink"
                }
            }),
        )
        .is_none()
    );
}

#[test]
fn derives_inactive_roster_membership_state_from_status_or_role() {
    let member = TelegramChatMember {
        sender_id: "user:42".to_owned(),
        sender_display_name: None,
        message_count: 0,
        last_message_at: None,
        source: "tdlib".to_owned(),
        provider_member_id: "user:42".to_owned(),
        username: None,
        role: Some("member".to_owned()),
        status: Some("left".to_owned()),
        is_admin: false,
        is_owner: false,
        permissions: json!({}),
        observed_at: None,
    };
    assert_eq!(inactive_roster_membership_state(&member), Some("left"));

    let banned = TelegramChatMember {
        status: Some("administrator".to_owned()),
        role: Some("banned".to_owned()),
        ..member
    };
    assert_eq!(inactive_roster_membership_state(&banned), Some("banned"));
}
