mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, delete_request_with_token, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;
#[tokio::test]
async fn telegram_dialog_actions_record_durable_command_rows() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-actions-{suffix}");
    let chat_id = format!("dialog-chat-{suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/telegram/accounts/fixture",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Actions",
            "external_account_id": format!("tg-dialog-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/telegram/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": chat_id,
            "provider_message_id": format!("dialog-message-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Dialog Action Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "@hermes dialog action command rows should exist.",
            "import_batch_id": format!("telegram-dialog-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let chats_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/telegram/chats?account_id={account_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chats response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    let telegram_chat_id = chats_body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();
    assert_eq!(chats_body["items"][0]["metadata"]["unread_count"], json!(1));
    assert_eq!(
        chats_body["items"][0]["metadata"]["mention_count"],
        json!(1)
    );

    let pin_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/telegram/chats/{telegram_chat_id}/pin"),
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("pin response");
    assert_eq!(pin_response.status(), StatusCode::OK);

    assert_ok(
        app.clone(),
        "/api/v1/telegram/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": chat_id,
            "provider_message_id": format!("dialog-message-follow-up-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Dialog Action Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Unread counters should survive repeated ingest.",
            "import_batch_id": format!("telegram-dialog-fixture-follow-up-{suffix}"),
            "occurred_at": "2026-06-06T12:05:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let detail_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/telegram/chats/{telegram_chat_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat detail response");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = json_body(detail_response).await;
    assert_eq!(detail_body["item"]["metadata"]["is_pinned"], json!(true));
    assert_eq!(detail_body["item"]["metadata"]["unread_count"], json!(2));
    assert_eq!(detail_body["item"]["metadata"]["mention_count"], json!(1));

    for action in [
        "unpin",
        "archive",
        "unarchive",
        "mute",
        "unmute",
        "read",
        "unread",
    ] {
        let response = app
            .clone()
            .oneshot(json_post_request_with_actor(
                &format!("/api/v1/telegram/chats/{telegram_chat_id}/{action}"),
                json!({
                    "account_id": account_id,
                    "provider_chat_id": chat_id
                }),
                LOCAL_API_TOKEN,
            ))
            .await
            .expect("dialog action response");
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "expected {action} to succeed"
        );
        if action == "read" {
            let body = json_body(response).await;
            assert_eq!(body["metadata"]["unread_count"], json!(0));
            assert_eq!(body["metadata"]["mention_count"], json!(0));
        } else if action == "unread" {
            let body = json_body(response).await;
            assert_eq!(body["metadata"]["unread_count"], json!(2));
            assert_eq!(body["metadata"]["mention_count"], json!(1));
        }
    }

    let commands_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/telegram/commands?account_id={account_id}&limit=20"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("commands response");
    assert_eq!(commands_response.status(), StatusCode::OK);
    let commands_body = json_body(commands_response).await;
    let items = commands_body["items"].as_array().expect("command items");
    let kinds: Vec<&str> = items
        .iter()
        .filter_map(|item| item["command_kind"].as_str())
        .collect();

    for expected_kind in [
        "pin",
        "unpin",
        "archive",
        "unarchive",
        "mute",
        "unmute",
        "mark_read",
        "mark_unread",
    ] {
        assert!(
            kinds.iter().any(|kind| kind == &expected_kind),
            "expected command row for {expected_kind}, got {kinds:?}"
        );
    }
    for provider_write_kind in [
        "archive",
        "unarchive",
        "mute",
        "unmute",
        "mark_read",
        "mark_unread",
    ] {
        let command = items
            .iter()
            .find(|item| item["command_kind"] == json!(provider_write_kind))
            .expect("provider-write dialog command row");
        assert_eq!(command["action_class"], json!("provider_write"));
    }

    let command_event_payload: Value = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'telegram.command.status_changed'
          AND payload->>'telegram_chat_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("dialog command event payload");
    assert_eq!(command_event_payload["action"], json!("mark_unread"));
    assert_eq!(command_event_payload["status"], json!("queued"));
    assert_eq!(
        command_event_payload["chat"]["telegram_chat_id"],
        json!(telegram_chat_id)
    );
    assert_eq!(
        command_event_payload["chat"]["provider_chat_id"],
        json!(chat_id)
    );
    assert_eq!(
        command_event_payload["chat"]["metadata"]["unread_count"],
        json!(2)
    );
    assert_eq!(
        command_event_payload["chat"]["metadata"]["mention_count"],
        json!(1)
    );
    assert_eq!(
        command_event_payload["chat"]["metadata"]["is_muted"],
        json!(false)
    );

    let chat_flag_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE event_type IN (
            'telegram.chat.pinned',
            'telegram.chat.archived',
            'telegram.chat.muted'
        )
          AND payload->>'telegram_chat_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_all(&pool)
    .await
    .expect("dialog chat flag events");
    assert_eq!(chat_flag_events.len(), 6);
    assert_eq!(chat_flag_events[0].0, "telegram.chat.pinned");
    assert_eq!(chat_flag_events[0].1["is_pinned"], json!(true));
    assert_eq!(chat_flag_events[1].0, "telegram.chat.pinned");
    assert_eq!(chat_flag_events[1].1["is_pinned"], json!(false));
    assert_eq!(chat_flag_events[2].0, "telegram.chat.archived");
    assert_eq!(chat_flag_events[2].1["is_archived"], json!(true));
    assert_eq!(chat_flag_events[3].0, "telegram.chat.archived");
    assert_eq!(chat_flag_events[3].1["is_archived"], json!(false));
    assert_eq!(chat_flag_events[4].0, "telegram.chat.muted");
    assert_eq!(chat_flag_events[4].1["is_muted"], json!(true));
    assert_eq!(chat_flag_events[5].0, "telegram.chat.muted");
    assert_eq!(chat_flag_events[5].1["is_muted"], json!(false));
    assert_eq!(
        chat_flag_events[5].1["chat"]["telegram_chat_id"],
        json!(telegram_chat_id)
    );
    assert_eq!(
        chat_flag_events[5].1["chat"]["provider_chat_id"],
        json!(chat_id)
    );
    assert_eq!(
        chat_flag_events[5].1["chat"]["metadata"]["is_muted"],
        json!(false)
    );

    let chat_updated_events: Vec<Value> = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'telegram.chat.updated'
          AND payload->>'telegram_chat_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_all(&pool)
    .await
    .expect("dialog chat update events");
    assert_eq!(chat_updated_events.len(), 2);
    assert_eq!(chat_updated_events[0]["action"], json!("mark_read"));
    assert_eq!(
        chat_updated_events[0]["chat"]["metadata"]["unread_count"],
        json!(0)
    );
    assert_eq!(chat_updated_events[1]["action"], json!("mark_unread"));
    assert_eq!(
        chat_updated_events[1]["chat"]["metadata"]["unread_count"],
        json!(2)
    );
    assert_eq!(
        chat_updated_events[1]["chat"]["metadata"]["mention_count"],
        json!(1)
    );
}
#[tokio::test]
async fn telegram_restore_and_reaction_actions_record_durable_command_rows() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-lifecycle-actions-{suffix}");
    let chat_id = format!("lifecycle-chat-{suffix}");
    let provider_message_id = format!("lifecycle-message-{suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/telegram/accounts/fixture",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Lifecycle Actions",
            "external_account_id": format!("tg-lifecycle-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "private",
                "chat_title": "Lifecycle Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Pavel Sidorov",
                "text": "Lifecycle actions should create durable command rows.",
                "import_batch_id": format!("telegram-lifecycle-fixture-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    let restore_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/telegram/messages/{message_id}/restore-visibility"),
            json!({
                "command_id": format!("restore-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "reason": "manual_restore"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("restore response");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let add_reaction_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/telegram/messages/{message_id}/reactions"),
            json!({
                "command_id": format!("react-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "reaction_emoji": "👍",
                "sender_id": "owner",
                "sender_display_name": "Owner"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("add reaction response");
    assert_eq!(add_reaction_response.status(), StatusCode::OK);

    let remove_reaction_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/telegram/messages/{message_id}/reactions?account_id={account_id}&provider_chat_id={chat_id}&provider_message_id={provider_message_id}&reaction_emoji=%F0%9F%91%8D&sender_id=owner&sender_display_name=Owner&command_id=unreact-{suffix}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("remove reaction response");
    assert_eq!(remove_reaction_response.status(), StatusCode::OK);

    let commands_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/telegram/commands?account_id={account_id}&limit=20"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("commands response");
    assert_eq!(commands_response.status(), StatusCode::OK);
    let commands_body = json_body(commands_response).await;
    let items = commands_body["items"].as_array().expect("command items");
    let kinds: Vec<&str> = items
        .iter()
        .filter_map(|item| item["command_kind"].as_str())
        .collect();

    for expected_kind in ["restore_visibility", "react", "unreact"] {
        assert!(
            kinds.iter().any(|kind| kind == &expected_kind),
            "expected command row for {expected_kind}, got {kinds:?}"
        );
    }
}
