use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::build_router_with_database;
use hermes_hub_backend::communications::{CommunicationProviderKind, ProviderAccountSecretPurpose};
use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::secrets::SecretKind;
use hermes_hub_backend::storage::Database;

const LOCAL_API_TOKEN: &str = "v4-api-test-token";
const LOCAL_API_ACTOR_ID: &str = "v4-api-test-client";
const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";

#[test]
fn v4_provider_and_secret_kinds_are_account_scoped() {
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
async fn v4_api_exercises_telegram_policy_and_call_foundation() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live V4 API smoke test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-{suffix}");
    let chat_id = format!("tg-chat-{suffix}");
    let policy_id = format!("policy-v4-{suffix}");
    let template_id = format!("template-v4-{suffix}");
    let call_id = format!("call-v4-{suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let capabilities_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v4/capabilities",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = json_body(capabilities_response).await;
    assert_eq!(capabilities_body["runtime_mode"], json!("fixture"));
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
            "/api/v4/telegram/accounts/fixture",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "V4 Telegram User",
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
            "/api/v4/telegram/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("tg-message-{suffix}"),
                "chat_kind": "private",
                "chat_title": "V4 Planning",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Maria Petrova",
                "text": "Please follow up on the V4 Telegram policy plan.",
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
            &format!("/api/v4/telegram/chats?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chats response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    assert_eq!(chats_body["items"][0]["provider_chat_id"], json!(chat_id));

    assert_ok(
        app.clone(),
        "/api/v4/policies/templates",
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
        "/api/v4/policies",
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

    let blocked = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v4/policies/telegram-send/dry-run",
            json!({
                "command_id": format!("dry-run-blocked-{suffix}"),
                "policy_id": policy_id,
                "provider_chat_id": format!("other-chat-{suffix}"),
                "variables": {"name": "Maria", "topic": "V4"},
                "source_context": {"source": "test"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("blocked dry-run");
    assert_eq!(blocked.status(), StatusCode::FORBIDDEN);

    let dry_run = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v4/policies/telegram-send/dry-run",
            json!({
                "command_id": format!("dry-run-allowed-{suffix}"),
                "policy_id": policy_id,
                "provider_chat_id": chat_id,
                "variables": {"name": "Maria", "topic": "V4"},
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

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_audit_log WHERE operation = 'automation.telegram_send.dry_run' AND actor_id = $1",
    )
    .bind(LOCAL_API_ACTOR_ID)
    .fetch_one(&pool)
    .await
    .expect("audit count");
    assert!(audit_count >= 1);

    assert_ok(
        app.clone(),
        "/api/v4/calls",
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
            &format!("/api/v4/calls/{call_id}/transcript"),
            json!({
                "transcript_id": format!("transcript-v4-{suffix}"),
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
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, LOCAL_API_ACTOR_ID)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, LOCAL_API_ACTOR_ID)
        .body(Body::empty())
        .expect("request")
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
