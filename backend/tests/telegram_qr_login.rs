mod telegram_support;

use std::env;

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use telegram_support::{
    LOCAL_API_TOKEN, delete_request_with_token, get_request_with_token, json_body,
    json_post_request_with_actor,
};

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
