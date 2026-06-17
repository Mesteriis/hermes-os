mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, ingest_fixture_telegram_message, json_body,
    json_post_request_with_actor, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn fixture_account_blocks_reply_and_forward_before_side_effects() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-reply-forward-gates-{suffix}");
    let provider_chat_id = format!("reply-forward-chat-{suffix}");
    let reply_target_provider_message_id = format!("{provider_chat_id}:root");
    let forward_source_provider_message_id = format!("{provider_chat_id}:forward-source");
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
            "display_name": "Telegram Reply Forward Gates",
            "external_account_id": format!("tg-reply-forward-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    let root_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &provider_chat_id,
        &reply_target_provider_message_id,
        "Root message for reply gate coverage.",
        "2026-06-06T12:00:00Z",
    )
    .await;
    let _forward_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &provider_chat_id,
        &forward_source_provider_message_id,
        "Source message for forward gate coverage.",
        "2026-06-06T12:01:00Z",
    )
    .await;

    let before_messages = message_count(app.clone(), &account_id, &provider_chat_id).await;

    let reply_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/telegram/messages/{root_message_id}/reply"),
            json!({
                "command_id": format!("reply-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "reply_to_provider_message_id": reply_target_provider_message_id,
                "text": "Reply should be blocked in fixture mode by capability gate."
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reply response");
    assert_eq!(reply_response.status(), StatusCode::BAD_REQUEST);

    let forward_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/telegram/messages/{root_message_id}/forward"),
            json!({
                "command_id": format!("forward-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "from_provider_chat_id": provider_chat_id,
                "from_provider_message_id": forward_source_provider_message_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("forward response");
    assert_eq!(forward_response.status(), StatusCode::BAD_REQUEST);

    let after_messages = message_count(app.clone(), &account_id, &provider_chat_id).await;
    assert_eq!(after_messages, before_messages);

    let send_audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE operation = 'telegram.message.send' AND metadata->>'account_id' = $1 AND metadata->>'provider_chat_id' = $2",
    )
    .bind(&account_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("send audit count");
    assert_eq!(send_audit_count, 0);

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.command.status_changed' AND payload->>'provider_chat_id' = $1",
    )
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("command event count");
    assert_eq!(command_event_count, 0);

    let created_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.message.created' AND payload->>'provider_chat_id' = $1 AND subject->>'id' <> $2",
    )
    .bind(&provider_chat_id)
    .bind(&root_message_id)
    .fetch_one(&pool)
    .await
    .expect("created event count");
    assert_eq!(created_event_count, 1);
}

async fn message_count<S>(app: S, account_id: &str, provider_chat_id: &str) -> usize
where
    S: tower::Service<axum::http::Request<axum::body::Body>, Response = axum::response::Response>
        + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/telegram/messages?account_id={account_id}&provider_chat_id={provider_chat_id}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("messages response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"].as_array().expect("message items").len()
}
