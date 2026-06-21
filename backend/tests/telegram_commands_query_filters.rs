mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::integrations::telegram::client::lifecycle;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, unique_suffix,
};
use testkit::context::TestContext;

#[tokio::test]
async fn telegram_commands_endpoint_filters_by_chat_and_kind() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-command-filter-{suffix}");
    let provider_chat_id = format!("command-filter-chat-{suffix}");
    let other_chat_id = format!("command-filter-other-{suffix}");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("HERMES_DEV_MODE", "true"),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Command Filter",
            "external_account_id": format!("tg-command-filter-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    insert_command(
        &pool,
        &account_id,
        "cmd-filter-mark-read",
        "mark_read",
        &provider_chat_id,
        &format!("{provider_chat_id}:1"),
    )
    .await;
    insert_command(
        &pool,
        &account_id,
        "cmd-filter-join",
        "join",
        &provider_chat_id,
        &format!("{provider_chat_id}:2"),
    )
    .await;
    insert_command(
        &pool,
        &account_id,
        "cmd-filter-other-chat",
        "mark_read",
        &other_chat_id,
        &format!("{other_chat_id}:1"),
    )
    .await;
    insert_command(
        &pool,
        &account_id,
        "cmd-filter-other-message",
        "mark_read",
        &provider_chat_id,
        &format!("{provider_chat_id}:99"),
    )
    .await;

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/integrations/telegram/commands?account_id={account_id}&provider_chat_id={provider_chat_id}&provider_message_id={provider_chat_id}:1&command_kinds=mark_read&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("commands response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("command items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["command_id"], json!("cmd-filter-mark-read"));
    assert_eq!(items[0]["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(items[0]["command_kind"], json!("mark_read"));
}

async fn insert_command(
    pool: &sqlx::PgPool,
    account_id: &str,
    command_id: &str,
    command_kind: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
) {
    lifecycle::insert_command(
        pool,
        command_id,
        account_id,
        command_kind,
        command_id,
        provider_chat_id,
        Some(provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        "hermes-frontend",
        json!({"source": "telegram_commands_query_filters"}),
        json!({"provider_chat_id": provider_chat_id}),
        json!({"source": "test"}),
    )
    .await
    .expect("insert command");
}
