use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
    NewRawCommunicationRecord, ProviderAccountSecretPurpose,
};
use hermes_hub_backend::domains::mail::messages::MessageProjectionStore;
use hermes_hub_backend::integrations::telegram::client::project_raw_telegram_message;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::secrets::{SecretKind, SecretReferenceStore, SecretStoreKind};
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-api-test-secret";

#[test]
fn telegram_provider_and_secret_kinds_are_account_scoped() {
    assert_eq!(
        CommunicationProviderKind::try_from("telegram_user").expect("telegram user"),
        CommunicationProviderKind::TelegramUser
    );
    assert_eq!(
        CommunicationProviderKind::try_from("telegram_bot").expect("telegram bot"),
        CommunicationProviderKind::TelegramBot
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramApiHash.accepts_secret_kind(SecretKind::ApiToken)
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramBotToken.accepts_secret_kind(SecretKind::ApiToken)
    );
    assert!(
        ProviderAccountSecretPurpose::TelegramSessionKey
            .accepts_secret_kind(SecretKind::PrivateKey)
    );
    assert!(
        !ProviderAccountSecretPurpose::TelegramBotToken.accepts_secret_kind(SecretKind::Password)
    );
}

#[tokio::test]
async fn telegram_api_exercises_policy_and_call_foundation() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live Telegram API smoke test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-{suffix}");
    let chat_id = format!("tg-chat-{suffix}");
    let policy_id = format!("policy-telegram-{suffix}");
    let template_id = format!("template-telegram-{suffix}");
    let call_id = format!("call-telegram-{suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let capabilities_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/telegram/capabilities",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = json_body(capabilities_response).await;
    assert_eq!(capabilities_body["runtime_mode"], json!("fixture"));
    assert_eq!(
        capabilities_body["telegram_app_credentials_configured"],
        json!(false)
    );
    assert_eq!(capabilities_body["qr_login_ready"], json!(false));
    assert_capability_status(
        &capabilities_body,
        "telegram_fixture_runtime",
        "available",
        true,
    );
    assert_capability_status(&capabilities_body, "automation_dry_run", "available", true);
    assert_capability_status(&capabilities_body, "tdlib_live_runtime", "blocked", false);
    assert_capability_status(&capabilities_body, "automation_live_send", "blocked", false);
    assert_capability_status(
        &capabilities_body,
        "whisper_rs_speech_to_text",
        "blocked",
        false,
    );
    assert!(
        capabilities_body["unsupported_features"]
            .as_array()
            .expect("unsupported features")
            .iter()
            .any(|feature| feature == "video_calls")
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/accounts/fixture",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram User",
                "external_account_id": format!("tg-user-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);
    let account_body = json_body(account_response).await;
    assert_eq!(account_body["provider_kind"], json!("telegram_user"));
    assert_eq!(account_body["runtime"], json!("fixture"));

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("tg-message-{suffix}"),
                "chat_kind": "private",
                "chat_title": "Telegram Planning",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Maria Petrova",
                "text": "Please follow up on the Telegram policy plan.",
                "import_batch_id": format!("telegram-fixture-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    assert!(
        message_body["message_id"]
            .as_str()
            .expect("message id")
            .starts_with("message:v4:telegram:")
    );

    let chats_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/telegram/chats?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chats response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    assert_eq!(chats_body["items"][0]["provider_chat_id"], json!(chat_id));

    assert_ok(
        app.clone(),
        "/api/v1/policies/templates",
        json!({
            "template_id": template_id,
            "name": "Follow up",
            "body_template": "Hi {{name}}, I will follow up on {{topic}}.",
            "required_variables": ["name", "topic"]
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/policies",
        json!({
            "policy_id": policy_id,
            "template_id": template_id,
            "name": "Allowed Telegram follow up",
            "enabled": true,
            "account_id": account_id,
            "allowed_chat_ids": [chat_id],
            "trigger_kind": "ai_follow_up",
            "max_sends_per_hour": 3,
            "quiet_hours": {},
            "conditions": {"source": "test"}
        }),
    )
    .await;

    let blocked_command_id = format!("dry-run-blocked-{suffix}");
    let blocked_chat_id = format!("other-chat-{suffix}");
    let blocked = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/policies/telegram-send/dry-run",
            json!({
                "command_id": blocked_command_id,
                "policy_id": policy_id,
                "provider_chat_id": blocked_chat_id,
                "variables": {"name": "Maria", "topic": "Telegram"},
                "source_context": {"source": "test"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("blocked dry-run");
    assert_eq!(blocked.status(), StatusCode::FORBIDDEN);

    let rejected_audit = sqlx::query(
        r#"
        SELECT target_kind, target_id, metadata
        FROM api_audit_log
        WHERE operation = 'automation.telegram_send.dry_run'
          AND actor_id = $1
          AND metadata->>'decision' = 'rejected'
          AND metadata->>'provider_chat_id' = $2
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind("hermes-frontend")
    .bind(&blocked_chat_id)
    .fetch_one(&pool)
    .await
    .expect("rejected dry-run audit");
    let rejected_target_kind: String = rejected_audit.try_get("target_kind").expect("target kind");
    let rejected_target_id: Option<String> =
        rejected_audit.try_get("target_id").expect("target id");
    let rejected_metadata: Value = rejected_audit.try_get("metadata").expect("metadata");
    assert_eq!(rejected_target_kind, "telegram_send_request");
    assert_eq!(
        rejected_target_id.as_deref(),
        Some(blocked_command_id.as_str())
    );
    assert_eq!(rejected_metadata["action_class"], json!("automation"));
    assert_eq!(rejected_metadata["capability"], json!("telegram.send"));
    assert_eq!(rejected_metadata["decision"], json!("rejected"));
    assert_eq!(
        rejected_metadata["reason"],
        json!("provider_chat_not_allowed")
    );
    assert_eq!(rejected_metadata["confirmation_required"], json!(true));
    assert_eq!(rejected_metadata["scoped_automation_policy"], json!(false));
    assert_eq!(rejected_metadata["automation_policy_id"], json!(policy_id));
    assert!(rejected_metadata.get("variables").is_none());
    assert!(rejected_metadata.get("source_context").is_none());
    assert!(rejected_metadata.get("rendered_text").is_none());
    assert!(rejected_metadata.get("rendered_preview_hash").is_none());

    let dry_run = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/policies/telegram-send/dry-run",
            json!({
                "command_id": format!("dry-run-allowed-{suffix}"),
                "policy_id": policy_id,
                "provider_chat_id": chat_id,
                "variables": {"name": "Maria", "topic": "Telegram"},
                "source_context": {"source": "test"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dry-run response");
    assert_eq!(dry_run.status(), StatusCode::OK);
    let dry_run_body = json_body(dry_run).await;
    assert_eq!(dry_run_body["status"], json!("allowed"));
    assert!(
        dry_run_body["rendered_preview_hash"]
            .as_str()
            .expect("hash")
            .starts_with("sha256:")
    );
    let outbound_message_id = dry_run_body["outbound_message_id"]
        .as_str()
        .expect("outbound message id");

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_audit_log WHERE operation = 'automation.telegram_send.dry_run' AND actor_id = $1",
    )
    .bind("hermes-frontend")
    .fetch_one(&pool)
    .await
    .expect("audit count");
    assert!(audit_count >= 2);

    let allowed_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'automation.telegram_send.dry_run'
          AND actor_id = $1
          AND target_id = $2
          AND metadata->>'decision' = 'allowed'
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind("hermes-frontend")
    .bind(outbound_message_id)
    .fetch_one(&pool)
    .await
    .expect("allowed dry-run audit metadata");
    assert_eq!(allowed_metadata["action_class"], json!("automation"));
    assert_eq!(allowed_metadata["capability"], json!("telegram.send"));
    assert_eq!(allowed_metadata["decision"], json!("allowed"));
    assert_eq!(
        allowed_metadata["reason"],
        json!("scoped_automation_policy_authorized")
    );
    assert_eq!(allowed_metadata["confirmation_required"], json!(false));
    assert_eq!(allowed_metadata["scoped_automation_policy"], json!(true));
    assert_eq!(allowed_metadata["automation_policy_id"], json!(policy_id));
    assert_eq!(
        allowed_metadata["rendered_preview_hash"],
        dry_run_body["rendered_preview_hash"]
    );
    assert!(allowed_metadata.get("variables").is_none());
    assert!(allowed_metadata.get("source_context").is_none());
    assert!(allowed_metadata.get("rendered_text").is_none());

    assert_ok(
        app.clone(),
        "/api/v1/calls",
        json!({
            "call_id": call_id,
            "account_id": account_id,
            "provider_call_id": format!("provider-call-{suffix}"),
            "provider_chat_id": chat_id,
            "direction": "incoming",
            "call_state": "ended",
            "started_at": "2026-06-06T12:10:00Z",
            "ended_at": "2026-06-06T12:20:00Z",
            "transcription_policy_id": policy_id,
            "metadata": {"runtime": "fixture"}
        }),
    )
    .await;

    let transcript_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/calls/{call_id}/transcript"),
            json!({
                "transcript_id": format!("transcript-telegram-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "source_audio_ref": format!("local-audio-{suffix}.wav"),
                "language_code": "en",
                "always_on_policy": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("transcript response");
    assert_eq!(transcript_response.status(), StatusCode::OK);
    let transcript_body = json_body(transcript_response).await;
    assert_eq!(transcript_body["transcript_status"], json!("succeeded"));
    assert_eq!(transcript_body["stt_provider"], json!("fixture-stt"));
    assert!(
        transcript_body["transcript_text"]
            .as_str()
            .expect("transcript text")
            .contains("follow up")
    );
}

#[tokio::test]
async fn telegram_fixture_runtime_status_can_start_account_actor() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-runtime-{suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/accounts/fixture",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Runtime",
                "external_account_id": format!("tg-runtime-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let initial_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("initial runtime status");
    assert_eq!(initial_status.status(), StatusCode::OK);
    let initial_body = json_body(initial_status).await;
    assert_eq!(initial_body["account_id"], json!(account_id));
    assert_eq!(initial_body["provider_kind"], json!("telegram_user"));
    assert_eq!(initial_body["runtime_kind"], json!("fixture"));
    assert_eq!(initial_body["status"], json!("stopped"));
    assert_eq!(initial_body["live_send_available"], json!(false));
    assert_eq!(initial_body["fixture_runtime"], json!(true));

    let start_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/runtime/start",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    assert_eq!(start_body["account_id"], json!(account_id));
    assert_eq!(start_body["status"], json!("running"));
    assert_eq!(start_body["runtime_kind"], json!("fixture"));

    let running_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("running runtime status");
    assert_eq!(running_status.status(), StatusCode::OK);
    let running_body = json_body(running_status).await;
    assert_eq!(running_body["status"], json!("running"));
    assert_eq!(running_body["last_error"], Value::Null);
}

#[tokio::test]
async fn telegram_manual_send_records_sent_message_and_redacted_provider_write_audit() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-send-{suffix}");
    let chat_id = format!("send-chat-{suffix}");
    let command_id = format!("manual-send-{suffix}");
    let message_text = "Manual Telegram reply from Hermes.";
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
            "display_name": "Telegram Send",
            "external_account_id": format!("tg-send-{suffix}"),
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
            "provider_message_id": format!("incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Manual Send Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Can Hermes reply here?",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let send_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/messages/send",
            json!({
                "command_id": command_id,
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "text": message_text
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("send response");
    assert_eq!(send_response.status(), StatusCode::OK);
    let send_body = json_body(send_response).await;
    assert_eq!(send_body["account_id"], json!(account_id));
    assert_eq!(send_body["provider_chat_id"], json!(chat_id));
    assert_eq!(send_body["delivery_state"], json!("sent"));
    assert_eq!(send_body["status"], json!("sent"));
    assert_eq!(send_body["runtime_kind"], json!("fixture"));
    assert!(
        send_body["message_id"]
            .as_str()
            .expect("message id")
            .starts_with("message:v4:telegram:")
    );
    assert!(
        send_body["rendered_preview_hash"]
            .as_str()
            .expect("preview hash")
            .starts_with("sha256:")
    );

    let messages_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/telegram/messages?account_id={account_id}&provider_chat_id={chat_id}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("messages response");
    assert_eq!(messages_response.status(), StatusCode::OK);
    let messages_body = json_body(messages_response).await;
    let sent_message = messages_body["items"]
        .as_array()
        .expect("messages")
        .iter()
        .find(|message| message["delivery_state"] == "sent")
        .expect("sent message");
    assert_eq!(sent_message["text"], json!(message_text));
    assert_eq!(sent_message["sender_display_name"], json!("Hermes"));

    let audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.message.send'
          AND actor_id = $1
          AND target_id = $2
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind("hermes-frontend")
    .bind(send_body["message_id"].as_str().expect("message id"))
    .fetch_one(&pool)
    .await
    .expect("manual send audit metadata");
    assert_eq!(audit_metadata["action_class"], json!("provider_write"));
    assert_eq!(audit_metadata["capability"], json!("telegram.message.send"));
    assert_eq!(audit_metadata["decision"], json!("allowed"));
    assert_eq!(
        audit_metadata["reason"],
        json!("explicit_user_confirmation")
    );
    assert_eq!(audit_metadata["confirmation_required"], json!(false));
    assert_eq!(audit_metadata["account_id"], json!(account_id));
    assert_eq!(audit_metadata["provider_chat_id"], json!(chat_id));
    assert_eq!(
        audit_metadata["rendered_preview_hash"],
        send_body["rendered_preview_hash"]
    );
    assert!(audit_metadata.get("text").is_none());
    assert!(audit_metadata.get("message_text").is_none());
    assert!(audit_metadata.get("rendered_text").is_none());
}

#[tokio::test]
async fn telegram_fixture_sync_chats_returns_account_chat_metadata() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-sync-chats-{suffix}");
    let chat_id = format!("sync-chat-{suffix}");
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
            "display_name": "Telegram Sync",
            "external_account_id": format!("tg-sync-{suffix}"),
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
            "provider_message_id": format!("incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Selected Sync Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Fixture chat metadata should sync.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/sync/chats",
            json!({
                "account_id": account_id,
                "limit": 25
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat sync response");

    assert_eq!(sync_response.status(), StatusCode::OK);
    let sync_body = json_body(sync_response).await;
    assert_eq!(sync_body["account_id"], json!(account_id));
    assert_eq!(sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(sync_body["status"], json!("synced"));
    assert_eq!(sync_body["synced_count"], json!(1));
    assert_eq!(sync_body["items"][0]["provider_chat_id"], json!(chat_id));
    assert_eq!(sync_body["items"][0]["title"], json!("Selected Sync Chat"));
}

#[tokio::test]
async fn telegram_fixture_sync_selected_history_returns_projected_messages() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-sync-history-{suffix}");
    let selected_chat_id = format!("selected-chat-{suffix}");
    let other_chat_id = format!("other-chat-{suffix}");
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
            "display_name": "Telegram History Sync",
            "external_account_id": format!("tg-history-{suffix}"),
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
            "provider_chat_id": selected_chat_id,
            "provider_message_id": format!("selected-incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Selected History Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Selected chat history message.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/telegram/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": other_chat_id,
            "provider_message_id": format!("other-incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Other History Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "This other chat must not be returned.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:01:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/sync/history",
            json!({
                "account_id": account_id,
                "provider_chat_id": selected_chat_id,
                "limit": 50
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("history sync response");

    assert_eq!(sync_response.status(), StatusCode::OK);
    let sync_body = json_body(sync_response).await;
    assert_eq!(sync_body["account_id"], json!(account_id));
    assert_eq!(sync_body["provider_chat_id"], json!(selected_chat_id));
    assert_eq!(sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(sync_body["status"], json!("synced"));
    assert_eq!(sync_body["synced_count"], json!(1));
    assert_eq!(
        sync_body["items"][0]["provider_chat_id"],
        json!(selected_chat_id)
    );
    assert_eq!(
        sync_body["items"][0]["text"],
        json!("Selected chat history message.")
    );
}

#[tokio::test]
async fn telegram_tdlib_projection_accepts_media_message_without_text() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("telegram-empty-media-{suffix}");
    let provider_chat_id = format!("-100{suffix}");
    let provider_message_id = format!("{provider_chat_id}:87403003904");

    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                "Telegram Empty Media",
                format!("tg-empty-media-{suffix}"),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw:telegram-empty-media:{suffix}"),
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:{suffix}"),
                format!("telegram-tdlib-history:{account_id}:{provider_chat_id}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": "Media Channel",
                    "chat_kind": "channel",
                    "sender_id": format!("chat:{provider_chat_id}"),
                    "sender_display_name": "Media Channel",
                    "text": "",
                    "delivery_state": "received",
                    "tdlib_raw": {
                        "@type": "message",
                        "id": 87403003904_i64,
                        "chat_id": provider_chat_id,
                        "content": {"@type": "messagePhoto"}
                    }
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "runtime": "tdlib",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw source");

    let projected = project_raw_telegram_message(&message_store, &raw)
        .await
        .expect("project empty media message");

    assert_eq!(projected.provider_record_id, provider_message_id);
    assert_eq!(projected.conversation_id, Some(provider_chat_id));
    assert_eq!(projected.body_text, "");
    assert_eq!(
        projected.message_metadata["tdlib_raw"]["content"]["@type"],
        json!("messagePhoto")
    );
}

#[tokio::test]
async fn telegram_fixture_media_download_fails_closed_without_live_runtime() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-{suffix}");
    let chat_id = format!("media-chat-{suffix}");
    let provider_message_id = format!("media-message-{suffix}");
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
            "display_name": "Telegram Media",
            "external_account_id": format!("tg-media-{suffix}"),
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
            "provider_message_id": provider_message_id,
            "chat_kind": "private",
            "chat_title": "Media Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Document metadata only.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let download_response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/media/download",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "tdlib_file_id": 42,
                "provider_attachment_id": "tdlib-file:42",
                "filename": "document.pdf",
                "content_type": "application/pdf"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("download response");

    assert_eq!(download_response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(download_response).await;
    assert_eq!(body["error"], json!("invalid_telegram_request"));
    assert!(
        body["message"]
            .as_str()
            .expect("message")
            .contains("Telegram media downloads require an enabled TDLib actor")
    );
}

#[tokio::test]
async fn telegram_live_account_setup_stores_bot_token_in_host_vault() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-bot-{suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("HERMES_DEV_MODE", "true"),
            (
                "HERMES_VAULT_HOME",
                vault_dir.path().join("vault").to_str().expect("vault path"),
            ),
            (
                "HERMES_DEV_KEY_PATH",
                vault_dir
                    .path()
                    .join("dev")
                    .join("master.key")
                    .to_str()
                    .expect("dev key path"),
            ),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let entropy_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);
    let create_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/vault/create",
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_bot",
                "display_name": "Telegram Bot",
                "external_account_id": format!("@hermes_bot_{suffix}"),
                "bot_token": "123456:telegram-bot-token",
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["provider_kind"], json!("telegram_bot"));
    assert_eq!(body["runtime"], json!("live_blocked"));
    assert_eq!(
        body["credential_bindings"][0]["secret_purpose"],
        json!("telegram_bot_token")
    );
    assert_eq!(
        body["credential_bindings"][0]["secret_kind"],
        json!("api_token")
    );
    assert_eq!(
        body["credential_bindings"][0]["store_kind"],
        json!("host_vault")
    );

    let account = sqlx::query(
        "SELECT provider_kind, config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account");
    let provider_kind: String = account.try_get("provider_kind").expect("provider kind");
    let config: Value = account.try_get("config").expect("config");
    assert_eq!(provider_kind, "telegram_bot");
    assert_eq!(config["runtime"], json!("live_blocked"));
    assert_eq!(config["transcription_enabled"], json!(false));
    assert!(config.get("bot_token").is_none());
    assert!(config.get("api_hash").is_none());

    let secret_ref = body["credential_bindings"][0]["secret_ref"]
        .as_str()
        .expect("secret ref");
    let secret_store = SecretReferenceStore::new(pool.clone());
    let reference = secret_store
        .secret_reference(secret_ref)
        .await
        .expect("secret reference query")
        .expect("secret reference exists");
    assert_eq!(reference.secret_kind, SecretKind::ApiToken);
    assert_eq!(reference.store_kind, SecretStoreKind::HostVault);
    assert_eq!(reference.metadata["provider"], json!("telegram_bot"));
    assert_eq!(reference.metadata["account_id"], json!(account_id));

    let database_payload_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM encrypted_secret_vault_entries WHERE secret_ref = $1",
    )
    .bind(secret_ref)
    .fetch_one(&pool)
    .await
    .expect("database payload count");
    assert_eq!(database_payload_count, 0);
}

#[tokio::test]
async fn telegram_qr_authorized_account_setup_persists_metadata_without_host_vault_secret() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-{suffix}");
    let tdlib_data_path = format!("docker/data/telegram/{account_id}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("HERMES_DEV_MODE", "true"),
            (
                "HERMES_VAULT_HOME",
                vault_dir.path().join("vault").to_str().expect("vault path"),
            ),
            (
                "HERMES_DEV_KEY_PATH",
                vault_dir
                    .path()
                    .join("dev")
                    .join("master.key")
                    .to_str()
                    .expect("dev key path"),
            ),
            ("DATABASE_URL", database_url.as_str()),
            ("HERMES_TELEGRAM_API_ID", "12345"),
            ("HERMES_TELEGRAM_API_HASH", "telegram-api-hash"),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "@second_account",
                "external_account_id": format!("telegram:{suffix}"),
                "tdlib_data_path": tdlib_data_path,
                "transcription_enabled": false,
                "qr_authorized": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["provider_kind"], json!("telegram_user"));
    assert_eq!(body["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(body["credential_bindings"], json!([]));

    let account = sqlx::query(
        "SELECT provider_kind, display_name, external_account_id, config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account");
    let provider_kind: String = account.try_get("provider_kind").expect("provider kind");
    let display_name: String = account.try_get("display_name").expect("display name");
    let external_account_id: String = account
        .try_get("external_account_id")
        .expect("external account id");
    let config: Value = account.try_get("config").expect("config");
    assert_eq!(provider_kind, "telegram_user");
    assert_eq!(display_name, "@second_account");
    assert_eq!(external_account_id, format!("telegram:{suffix}"));
    assert_eq!(config["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(config["tdlib_data_path"], json!(tdlib_data_path));
    assert_eq!(config["transcription_enabled"], json!(false));
    assert!(config.get("api_hash").is_none());
    assert!(config.get("bot_token").is_none());

    let binding_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM communication_provider_account_secret_refs WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("binding count");
    assert_eq!(binding_count, 0);
}

#[tokio::test]
async fn telegram_finalized_qr_account_setup_infers_qr_authorized_runtime() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-inferred-{suffix}");
    let tdlib_data_path = format!("docker/data/telegram/{account_id}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("HERMES_DEV_MODE", "true"),
            (
                "HERMES_VAULT_HOME",
                vault_dir.path().join("vault").to_str().expect("vault path"),
            ),
            (
                "HERMES_DEV_KEY_PATH",
                vault_dir
                    .path()
                    .join("dev")
                    .join("master.key")
                    .to_str()
                    .expect("dev key path"),
            ),
            ("DATABASE_URL", database_url.as_str()),
            ("HERMES_TELEGRAM_API_ID", "12345"),
            ("HERMES_TELEGRAM_API_HASH", "telegram-api-hash"),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "@inferred_qr",
                "external_account_id": format!("telegram:{suffix}"),
                "tdlib_data_path": tdlib_data_path,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(body["credential_bindings"], json!([]));

    let config: Value = sqlx::query_scalar(
        "SELECT config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account config");
    assert_eq!(config["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(config["tdlib_data_path"], json!(tdlib_data_path));
    assert!(config.get("api_hash").is_none());

    let binding_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM communication_provider_account_secret_refs WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("binding count");
    assert_eq!(binding_count, 0);
}

#[tokio::test]
async fn telegram_live_account_setup_api_requires_configured_database() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/accounts",
            json!({
                "account_id": "telegram-no-db",
                "provider_kind": "telegram_bot",
                "display_name": "Telegram No DB",
                "external_account_id": "@telegram_no_db",
                "bot_token": "123456:telegram-bot-token"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
}

#[tokio::test]
async fn telegram_capabilities_report_qr_login_readiness_inputs() {
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            (
                "HERMES_TDJSON_PATH",
                "/tmp/hermes-hub-test-missing-libtdjson.dylib",
            ),
            ("HERMES_TELEGRAM_API_ID", "12345"),
            ("HERMES_TELEGRAM_API_HASH", "telegram-api-hash"),
        ])
        .expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/telegram/capabilities",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("capabilities response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["telegram_app_credentials_configured"], json!(true));
    assert_eq!(body["tdjson_runtime_available"], json!(false));
    assert_eq!(body["qr_login_ready"], json!(false));
    assert_capability_status(&body, "tdlib_live_runtime", "blocked", false);
}

#[tokio::test]
async fn telegram_qr_login_start_reports_tdlib_runtime_unavailable() {
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            (
                "HERMES_TDJSON_PATH",
                "/tmp/hermes-hub-test-missing-libtdjson.dylib",
            ),
        ])
        .expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/login/qr/start",
            json!({
                "account_id": "telegram-qr",
                "display_name": "Telegram QR",
                "external_account_id": "qr-login:telegram-qr",
                "api_id": 12345,
                "api_hash": "telegram-api-hash",
                "session_encryption_key": "telegram-session-key",
                "tdlib_data_path": "docker/data/telegram/telegram-qr",
                "transcription_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR login response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_tdlib_runtime_unavailable"));
}

#[tokio::test]
async fn telegram_qr_login_start_uses_configured_app_credentials_when_payload_omits_them() {
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            (
                "HERMES_TDJSON_PATH",
                "/tmp/hermes-hub-test-missing-libtdjson.dylib",
            ),
            ("HERMES_TELEGRAM_API_ID", "12345"),
            ("HERMES_TELEGRAM_API_HASH", "telegram-api-hash"),
        ])
        .expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/login/qr/start",
            json!({
                "account_id": "telegram-qr-configured",
                "display_name": "Telegram QR Configured",
                "external_account_id": "qr-login:telegram-qr-configured",
                "session_encryption_key": "telegram-session-key",
                "tdlib_data_path": "docker/data/telegram/telegram-qr-configured",
                "transcription_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR login response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_tdlib_runtime_unavailable"));
}

#[tokio::test]
async fn telegram_live_smoke_syncs_configured_account_when_explicitly_enabled() {
    if env::var("HERMES_TELEGRAM_LIVE_SMOKE").ok().as_deref() != Some("1") {
        eprintln!("skipping live Telegram TDLib smoke test: HERMES_TELEGRAM_LIVE_SMOKE is not 1");
        return;
    }

    let database_url =
        env::var("HERMES_TEST_DATABASE_URL").expect("HERMES_TEST_DATABASE_URL must be set");
    let account_id = env::var("HERMES_TELEGRAM_LIVE_ACCOUNT_ID")
        .expect("HERMES_TELEGRAM_LIVE_ACCOUNT_ID must be set");
    let provider_chat_id =
        env::var("HERMES_TELEGRAM_LIVE_CHAT_ID").expect("HERMES_TELEGRAM_LIVE_CHAT_ID must be set");
    let local_api_secret =
        env::var("HERMES_LOCAL_API_SECRET").expect("HERMES_LOCAL_API_SECRET must be set");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(AppConfig::from_env().expect("config"), database);

    let start_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/runtime/start",
            json!({ "account_id": account_id }),
            &local_api_secret,
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    assert_eq!(start_body["account_id"], json!(account_id));
    assert_eq!(start_body["runtime_kind"], json!("tdlib_qr_authorized"));
    assert_eq!(start_body["status"], json!("running"));

    let history_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/sync/history",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "limit": 25
            }),
            &local_api_secret,
        ))
        .await
        .expect("history sync response");
    assert_eq!(history_response.status(), StatusCode::OK);
    let history_body = json_body(history_response).await;
    assert_eq!(history_body["account_id"], json!(account_id));
    assert_eq!(history_body["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(history_body["runtime_kind"], json!("tdlib_qr_authorized"));
    assert_eq!(history_body["status"], json!("synced"));
}

#[tokio::test]
async fn telegram_qr_login_status_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/telegram/login/qr/missing-setup",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR status response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}

#[tokio::test]
async fn telegram_qr_login_password_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/telegram/login/qr/missing-setup/password",
            json!({ "password": "test-password" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR password response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}

#[tokio::test]
async fn telegram_qr_login_cancel_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(delete_request_with_token(
            "/api/v1/telegram/login/qr/missing-setup",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR cancel response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}

fn assert_capability_status(body: &Value, capability: &str, status: &str, closure_gate: bool) {
    let capabilities = body["capabilities"].as_array().expect("capabilities");
    assert!(
        capabilities.iter().any(|item| {
            item["capability"] == capability
                && item["status"] == status
                && item["closure_gate"] == closure_gate
        }),
        "expected capability {capability} to have status {status} and closure_gate {closure_gate}"
    );
}

async fn assert_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post_request_with_actor(path, body, LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

fn json_post_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-hermes-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn delete_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(path)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn vault_entropy_events(count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            json!({
                "x": index % 997,
                "y": index % 577,
                "dx": (index % 11) as i64 - 5,
                "dy": (index % 13) as i64 - 6,
                "timestamp_ms": index * 5,
                "velocity": (index % 19) as f64 / 10.0,
                "acceleration": (index % 23) as f64 / 100.0,
                "interval_ms": 5
            })
        })
        .collect()
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    format!("{now}")
}
