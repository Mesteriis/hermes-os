use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, NewRawCommunicationRecord, ProviderAccountSecretPurpose,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use hermes_hub_backend::platform::storage::Database;
use sqlx::Row;
use testkit::context::TestContext;

const TOKEN: &str = "mail-account-management-test-token";

async fn app(ctx: &TestContext) -> axum::Router {
    let database = Database::connect(Some(&ctx.connection_string()))
        .await
        .expect("database");
    build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", TOKEN),
            ("DATABASE_URL", ctx.connection_string().as_str()),
        ])
        .expect("config"),
        database,
    )
}

fn request(method: Method, uri: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-hermes-secret", TOKEN);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("json")
}

#[tokio::test]
async fn email_account_management_lists_gets_exports_logs_out_and_deletes_unused_account() {
    let ctx = TestContext::new().await;
    let store = CommunicationIngestionStore::new(ctx.pool().clone());
    let secret_store = SecretReferenceStore::new(ctx.pool().clone());
    store
        .upsert_provider_account(
            &NewProviderAccount::new(
                "fastmail-primary",
                EmailProviderKind::Imap,
                "Fastmail",
                "alex@example.com",
            )
            .config(json!({
                "host": "imap.fastmail.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "alex@example.com",
                "smtp_host": "smtp.fastmail.com",
                "smtp_port": 587,
                "smtp_tls": true,
                "smtp_starttls": true,
                "provider_preset": "fastmail"
            })),
        )
        .await
        .expect("account");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            "secret:fastmail-primary:imap-password",
            SecretKind::AppPassword,
            SecretStoreKind::TestDouble,
            "Fastmail app password",
        ))
        .await
        .expect("secret reference");
    store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            "fastmail-primary",
            ProviderAccountSecretPurpose::ImapPassword,
            "secret:fastmail-primary:imap-password",
        ))
        .await
        .expect("bind provider secret");

    let app = app(&ctx).await;

    let response = app
        .clone()
        .oneshot(request(Method::GET, "/api/v1/email-accounts", None))
        .await
        .expect("list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(
        body["items"][0]["account"]["account_id"],
        "fastmail-primary"
    );
    assert_eq!(body["items"][0]["capabilities"]["send"], true);
    assert_eq!(body["items"][0]["capabilities"]["local_trash"], true);

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            "/api/v1/email-accounts/fastmail-primary",
            None,
        ))
        .await
        .expect("get response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account"]["external_account_id"], "alex@example.com");

    let response = app
        .clone()
        .oneshot(request(
            Method::GET,
            "/api/v1/email-accounts/fastmail-primary/export",
            None,
        ))
        .await
        .expect("export response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account"]["account_id"], "fastmail-primary");
    let serialized = body.to_string();
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("secret_ref"));
    assert!(!serialized.contains("token"));

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/email-accounts/fastmail-primary/logout",
            None,
        ))
        .await
        .expect("logout response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account"]["config"]["auth_state"], "logged_out");
    assert_eq!(body["sync_settings"]["sync_enabled"], false);

    let logout_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code, link.relationship_kind, observation.payload
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'vault'
           AND link.entity_kind = 'communication_provider_account'
           AND link.entity_id = 'fastmail-primary'
           AND link.relationship_kind = 'config_update'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .fetch_one(ctx.pool())
    .await
    .expect("logout config observation");
    assert_eq!(
        logout_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        logout_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION"
    );
    let logout_payload: Value = logout_observation.try_get("payload").expect("payload");
    assert_eq!(logout_payload["action"], "logout");
    assert_eq!(logout_payload["account_id"], "fastmail-primary");

    let response = app
        .clone()
        .oneshot(request(
            Method::DELETE,
            "/api/v1/email-accounts/fastmail-primary",
            None,
        ))
        .await
        .expect("delete response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["deleted"], true);
    assert_eq!(
        body["unbound_secret_refs"],
        json!(["secret:fastmail-primary:imap-password"])
    );

    let account_delete_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code, link.relationship_kind
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'vault'
           AND link.entity_kind = 'communication_provider_account'
           AND link.entity_id = 'fastmail-primary'
           AND link.relationship_kind = 'delete'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .fetch_one(ctx.pool())
    .await
    .expect("provider account delete observation");
    assert_eq!(
        account_delete_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        account_delete_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_ACCOUNT_DELETED"
    );

    let binding_remove_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code, link.relationship_kind
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'vault'
           AND link.entity_kind = 'communication_provider_secret_binding'
           AND link.entity_id = 'fastmail-primary:imap_password'
           AND link.relationship_kind = 'remove'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .fetch_one(ctx.pool())
    .await
    .expect("provider secret binding removal observation");
    assert_eq!(
        binding_remove_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        binding_remove_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED"
    );

    let response = app
        .oneshot(request(
            Method::GET,
            "/api/v1/email-accounts/fastmail-primary",
            None,
        ))
        .await
        .expect("get deleted response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn email_account_delete_rejects_accounts_with_retained_raw_records() {
    let ctx = TestContext::new().await;
    let store = CommunicationIngestionStore::new(ctx.pool().clone());
    store
        .upsert_provider_account(&NewProviderAccount::new(
            "imap-with-evidence",
            EmailProviderKind::Imap,
            "Evidence IMAP",
            "evidence@example.com",
        ))
        .await
        .expect("account");
    store
        .record_raw_source(&NewRawCommunicationRecord::new(
            "raw:mail-account-delete",
            "imap-with-evidence",
            "email",
            "provider:1",
            "sha256:test",
            "batch:test",
            json!({}),
        ))
        .await
        .expect("raw record");

    let app = app(&ctx).await;
    let response = app
        .oneshot(request(
            Method::DELETE,
            "/api/v1/email-accounts/imap-with-evidence",
            None,
        ))
        .await
        .expect("delete response");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(body["error"], "email_account_delete_conflict");
}

#[tokio::test]
async fn email_account_import_creates_sanitized_account_and_rejects_secrets() {
    let ctx = TestContext::new().await;
    let app = app(&ctx).await;

    let response = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/api/v1/email-accounts/import",
            Some(json!({
                "account": {
                    "account_id": "proton-bridge",
                    "provider_kind": "imap",
                    "display_name": "Proton Bridge",
                    "external_account_id": "alex@proton.me",
                    "config": {
                        "host": "127.0.0.1",
                        "port": 1143,
                        "tls": false,
                        "mailbox": "INBOX",
                        "provider_preset": "proton_bridge"
                    }
                },
                "sync_settings": {
                    "sync_enabled": false,
                    "batch_size": 25,
                    "poll_interval_seconds": 900
                }
            })),
        ))
        .await
        .expect("import response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account"]["account_id"], "proton-bridge");
    assert_eq!(body["sync_settings"]["sync_enabled"], false);
    assert_eq!(body["sync_settings"]["batch_size"], 25);

    let response = app
        .oneshot(request(
            Method::POST,
            "/api/v1/email-accounts/import",
            Some(json!({
                "account": {
                    "account_id": "bad-secret-import",
                    "provider_kind": "imap",
                    "display_name": "Bad Import",
                    "external_account_id": "bad@example.com",
                    "config": {
                        "host": "imap.example.com",
                        "password": "do-not-import"
                    },
                    "secret_ref": "secret:provider-account:bad-secret-import:imap_password"
                }
            })),
        ))
        .await
        .expect("secret import response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
}
