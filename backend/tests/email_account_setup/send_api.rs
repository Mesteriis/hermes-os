use axum::http::StatusCode;
use serde_json::json;
use tempfile::tempdir;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

use super::support::{
    LOCAL_API_TOKEN, MockSmtpServer, json_body, json_request_with_token_and_actor,
    unlock_test_vault,
};

#[tokio::test]
async fn imap_send_api_sends_via_configured_smtp_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config = AppConfig::from_pairs([
        ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
        ("HERMES_DEV_MODE", "true"),
        (
            "HERMES_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "HERMES_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
        ("DATABASE_URL", database_url.as_str()),
    ])
    .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let smtp_server = MockSmtpServer::start();
    let account_id = "imap-send-primary";
    let setup_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "imap",
                "display_name": "IMAP Send",
                "external_account_id": "sender@example.com",
                "host": "imap.example.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "sender@example.com",
                "password": "smtp-app-password",
                "secret_kind": "password",
                "smtp_host": smtp_server.addr().ip().to_string(),
                "smtp_port": smtp_server.addr().port(),
                "smtp_tls": false,
                "smtp_starttls": false,
                "smtp_username": "sender@example.com"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("setup response");
    assert_eq!(setup_response.status(), StatusCode::OK);

    let send_response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "cc": ["copy@example.com"],
                "subject": "SMTP send test",
                "body_text": "Message body from Hermes test.",
                "confirmed_provider_write": true
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("send response");
    assert_eq!(send_response.status(), StatusCode::OK);
    let send_body = json_body(send_response).await;
    assert_eq!(send_body["transport"], "smtp");
    assert_eq!(send_body["status"], "sent");
    assert_eq!(
        send_body["accepted_recipients"],
        json!(["recipient@example.com", "copy@example.com"])
    );

    let commands = smtp_server.commands();
    assert!(commands.iter().any(|line| line == "AUTH LOGIN"));
    assert!(
        commands
            .iter()
            .any(|line| line == "MAIL FROM:<sender@example.com>")
    );
    assert!(
        commands
            .iter()
            .any(|line| line == "RCPT TO:<recipient@example.com>")
    );
    assert!(
        commands
            .iter()
            .any(|line| line == "RCPT TO:<copy@example.com>")
    );
    assert!(commands.iter().any(|line| line == "DATA"));
}

#[tokio::test]
async fn gmail_send_api_is_explicitly_unsupported_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config = AppConfig::from_pairs([
        ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
        ("HERMES_DEV_MODE", "true"),
        (
            "HERMES_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "HERMES_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
        ("DATABASE_URL", database_url.as_str()),
    ])
    .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let account_id = "gmail-send-disabled";
    CommunicationIngestionStore::new(pool)
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                EmailProviderKind::Gmail,
                "Gmail Send Disabled",
                "sender@gmail.com",
            )
            .config(json!({
                "auth": "oauth",
                "api": "gmail"
            })),
        )
        .await
        .expect("store gmail account");

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Gmail send disabled",
                "body_text": "Gmail provider-side send is not enabled.",
                "confirmed_provider_write": true
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("send response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert_eq!(
        body["message"],
        "Gmail send is unavailable until OAuth send scopes are configured"
    );
}

#[tokio::test]
async fn imap_send_api_requires_smtp_password_binding_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config = AppConfig::from_pairs([
        ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
        ("HERMES_DEV_MODE", "true"),
        (
            "HERMES_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "HERMES_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
        ("DATABASE_URL", database_url.as_str()),
    ])
    .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let smtp_server = MockSmtpServer::start();
    let account_id = "imap-send-missing-smtp-password";
    let setup_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "imap",
                "display_name": "IMAP Missing SMTP Password",
                "external_account_id": "sender@example.com",
                "host": "imap.example.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "sender@example.com",
                "password": "smtp-app-password",
                "secret_kind": "password",
                "smtp_host": smtp_server.addr().ip().to_string(),
                "smtp_port": smtp_server.addr().port(),
                "smtp_tls": false,
                "smtp_starttls": false,
                "smtp_username": "sender@example.com"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("setup response");
    assert_eq!(setup_response.status(), StatusCode::OK);

    sqlx::query(
        "DELETE FROM communication_provider_account_secret_refs WHERE account_id = $1 AND secret_purpose = 'smtp_password'",
    )
    .bind(account_id)
    .execute(&pool)
    .await
    .expect("delete smtp binding");

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Missing SMTP binding",
                "body_text": "This must not reach SMTP.",
                "confirmed_provider_write": true
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("send response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert!(
        smtp_server
            .commands()
            .iter()
            .all(|line| !line.starts_with("MAIL FROM"))
    );
}

#[tokio::test]
async fn imap_send_api_does_not_send_when_audit_record_fails_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config = AppConfig::from_pairs([
        ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
        ("HERMES_DEV_MODE", "true"),
        (
            "HERMES_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "HERMES_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
        ("DATABASE_URL", database_url.as_str()),
    ])
    .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app.clone()).await;

    let smtp_server = MockSmtpServer::start();
    let account_id = "imap-send-audit-fail";
    let setup_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "imap",
                "display_name": "IMAP Audit Failure",
                "external_account_id": "sender@example.com",
                "host": "imap.example.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "sender@example.com",
                "password": "smtp-app-password",
                "secret_kind": "password",
                "smtp_host": smtp_server.addr().ip().to_string(),
                "smtp_port": smtp_server.addr().port(),
                "smtp_tls": false,
                "smtp_starttls": false,
                "smtp_username": "sender@example.com"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("setup response");
    assert_eq!(setup_response.status(), StatusCode::OK);

    sqlx::query("DROP TABLE api_audit_log")
        .execute(&pool)
        .await
        .expect("drop audit table");

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Audit fail closed",
                "body_text": "This must not reach SMTP.",
                "confirmed_provider_write": true
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("send response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        smtp_server
            .commands()
            .iter()
            .all(|line| !line.starts_with("MAIL FROM"))
    );
}
