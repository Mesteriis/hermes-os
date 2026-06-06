use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::build_router_with_database;
use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::settings::ApplicationSettingsStore;
use hermes_hub_backend::storage::Database;

const LOCAL_API_TOKEN: &str = "settings-api-test-token";
const LOCAL_API_ACTOR_ID: &str = "settings-api-test-client";
const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";

static SETTINGS_DB_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn application_settings_store_lists_seeded_settings_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live application settings store test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    let settings = store.list_settings().await.expect("list settings");

    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "ai.chat_model")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "server.http_addr")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.actor_id")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "ui.theme")
    );
    assert!(
        settings
            .iter()
            .all(|setting| !setting.setting_key.contains("password"))
    );
}

#[tokio::test]
async fn database_startup_repairs_declared_application_settings_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live application settings repair test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    sqlx::query("DELETE FROM application_settings WHERE setting_key = 'frontend.actor_id'")
        .execute(&pool)
        .await
        .expect("delete declared setting");
    sqlx::query(
        r#"
        UPDATE application_settings
        SET
            value = '"broken"'::jsonb,
            label = 'Broken density',
            metadata = '{}'::jsonb
        WHERE setting_key = 'ui.density'
        "#,
    )
    .execute(&pool)
    .await
    .expect("corrupt declared setting");
    sqlx::query(
        r#"
        INSERT INTO application_settings (
            setting_key,
            category,
            value_kind,
            value,
            label,
            description,
            metadata
        )
        VALUES (
            'custom.unexpected',
            'custom',
            'string',
            '"manual"'::jsonb,
            'Manual custom setting',
            'This row must not become a supported setting surface.',
            '{}'::jsonb
        )
        ON CONFLICT (setting_key) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert undeclared setting");

    drop(pool);
    drop(database);

    let repaired_database = Database::connect(Some(&database_url))
        .await
        .expect("database reconnect repairs settings");
    let store =
        ApplicationSettingsStore::new(repaired_database.pool().expect("configured pool").clone());
    let settings = store.list_settings().await.expect("list repaired settings");

    let actor_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.actor_id")
        .expect("frontend actor setting restored");
    assert_eq!(actor_setting.value, json!("desktop-shell"));
    assert_eq!(
        actor_setting.updated_by_actor_id.as_deref(),
        Some("system:settings_repair")
    );

    let density_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "ui.density")
        .expect("UI density setting restored");
    assert_eq!(density_setting.label, "UI density");
    assert_eq!(density_setting.value, json!("comfortable"));
    assert_eq!(
        density_setting.updated_by_actor_id.as_deref(),
        Some("system:settings_repair")
    );
    assert!(density_setting.metadata.get("allowed_values").is_some());
    assert!(
        settings
            .iter()
            .all(|setting| setting.setting_key != "custom.unexpected")
    );
}

#[tokio::test]
async fn application_settings_api_updates_existing_setting_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live application settings API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );
    let response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            "/api/v2/settings/ui.theme",
            json!({ "value": "dark" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["setting_key"], json!("ui.theme"));
    assert_eq!(body["value"], json!("dark"));
    assert_eq!(body["updated_by_actor_id"], json!(LOCAL_API_ACTOR_ID));

    let list_response = app
        .clone()
        .oneshot(get_request_with_token("/api/v2/settings", LOCAL_API_TOKEN))
        .await
        .expect("list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = json_body(list_response).await;
    let items = list_body["items"].as_array().expect("settings items");
    assert!(items.iter().any(|item| {
        item["setting_key"] == json!("ui.theme") && item["value"] == json!("dark")
    }));

    let _ = app
        .clone()
        .oneshot(json_put_request_with_actor(
            "/api/v2/settings/ui.theme",
            json!({ "value": "system" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("restore response");
}

#[tokio::test]
async fn application_settings_api_rejects_secret_like_setting_keys_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live application settings validation test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_put_request_with_actor(
            "/api/v2/settings/mail.password",
            json!({ "value": "not-allowed" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_application_setting"));
}

#[tokio::test]
async fn settings_accounts_api_lists_provider_accounts_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live settings accounts API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_settings_accounts_{suffix}");
    CommunicationIngestionStore::new(pool)
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Icloud,
            "Settings iCloud account",
            format!("settings-{suffix}@icloud.com"),
        ))
        .await
        .expect("seed provider account");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v2/settings/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("account items");
    assert!(items.iter().any(|item| {
        item["account_id"] == json!(account_id) && item["provider_kind"] == json!("icloud")
    }));
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, LOCAL_API_ACTOR_ID)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request_with_actor(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, LOCAL_API_ACTOR_ID)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
