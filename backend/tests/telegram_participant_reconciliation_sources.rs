use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::integrations::telegram::client::participants::reconcile_join_commands_from_provider_roster_with_source;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "telegram-participant-reconcile-source-secret";

#[tokio::test]
async fn telegram_basic_group_roster_reconciliation_records_observed_source() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-basic-group-reconcile-{suffix}");
    let provider_chat_id = format!("basic-group-reconcile-chat-{suffix}");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    post_ok(
        app.clone(),
        "/api/v1/telegram/accounts/fixture",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Basic Group Reconcile",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/telegram/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("basic-group-reconcile-message-{suffix}"),
            "chat_kind": "group",
            "chat_title": "Basic Group Reconcile Room",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": format!("telegram-basic-group-reconcile-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let join_body = command_response(
        app.clone(),
        "/api/v1/telegram/chats/join",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id
        }),
    )
    .await;
    let command_id = join_body["command_id"].as_str().expect("command id");

    let observed_at = Utc::now();
    let commands = reconcile_join_commands_from_provider_roster_with_source(
        &pool,
        &account_id,
        &provider_chat_id,
        "user:12345",
        observed_at,
        "tdlib.getBasicGroupFullInfo",
    )
    .await
    .expect("reconciled join commands");

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command_id, command_id);
    assert_eq!(commands[0].status, "completed");
    assert_eq!(commands[0].reconciliation_status, "observed");
    assert_eq!(
        commands[0].provider_state["observed_via"],
        "tdlib.getBasicGroupFullInfo"
    );
    assert_eq!(
        commands[0].result_payload["source"],
        "tdlib.getBasicGroupFullInfo"
    );

    let row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled command row");
    let provider_state: Value = row.try_get("provider_state").expect("provider state");
    let result_payload: Value = row.try_get("result_payload").expect("result payload");
    assert_eq!(
        row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        row.try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert_eq!(
        provider_state["observed_via"],
        "tdlib.getBasicGroupFullInfo"
    );
    assert_eq!(provider_state["membership_state"], "present");
    assert_eq!(result_payload["source"], "tdlib.getBasicGroupFullInfo");
}

async fn command_response<S>(app: S, path: &str, body: Value) -> Value
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post(path, body))
        .await
        .expect("command response");
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

async fn post_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post(path, body))
        .await
        .expect("post response");
    assert_eq!(response.status(), StatusCode::OK);
}

fn json_post(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-hermes-secret", LOCAL_API_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1_000_000)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
        .to_string()
}
