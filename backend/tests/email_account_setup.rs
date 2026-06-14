use std::env;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tempfile::tempdir;
use tokio::time::{Duration, sleep};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::calendar::events::CalendarAccountStore;
use hermes_hub_backend::domains::mail::accounts::{
    EmailAccountSetupService, GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
    ProviderAccountSecretPurpose,
};
use hermes_hub_backend::platform::config::{AppConfig, GoogleOAuthClientType};
use hermes_hub_backend::platform::secrets::DatabaseEncryptedSecretVault;
use hermes_hub_backend::platform::secrets::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReferenceStore, SecretResolver,
    SecretStoreKind,
};
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::vault::{HostVault, HostVaultConfig, SecretEntryContext};
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "account-setup-test-token";

#[test]
fn gmail_oauth_setup_defaults_to_mail_calendar_and_contacts_read_scopes() {
    let request = GmailOAuthSetupRequest::new(
        "acct_google_workspace",
        "Google Workspace",
        "",
        "desktop-client-id",
        "http://127.0.0.1:18088/oauth/callback",
    );

    assert_eq!(
        request.scopes,
        [
            "https://www.googleapis.com/auth/gmail.readonly",
            "https://www.googleapis.com/auth/calendar.readonly",
            "https://www.googleapis.com/auth/contacts.readonly",
        ]
    );
}

#[test]
fn app_config_accepts_google_oauth_client_credentials() {
    let config = AppConfig::from_pairs([
        ("HERMES_GOOGLE_OAUTH_CLIENT_ID", "google-client-id"),
        ("HERMES_GOOGLE_OAUTH_CLIENT_SECRET", "google-client-secret"),
    ])
    .expect("config");

    assert_eq!(config.google_oauth_client_id(), Some("google-client-id"));
    assert_eq!(
        config
            .google_oauth_client_secret()
            .expect("google client secret")
            .expose_for_runtime(),
        "google-client-secret"
    );
}

#[test]
fn app_config_accepts_google_oauth_installed_client_json() {
    let config = AppConfig::from_pairs([(
        "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
        r#"{
            "installed": {
                "client_id": "desktop-client-id.apps.googleusercontent.com",
                "project_id": "hermes-hub-local",
                "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                "token_uri": "https://oauth2.googleapis.com/token",
                "client_secret": "desktop-client-secret",
                "redirect_uris": ["http://localhost"]
            }
        }"#,
    )])
    .expect("config");

    let google_client = config
        .google_oauth_client()
        .expect("google oauth client config");
    assert_eq!(
        google_client.client_type(),
        GoogleOAuthClientType::Installed
    );
    assert_eq!(
        google_client.client_id(),
        "desktop-client-id.apps.googleusercontent.com"
    );
    assert_eq!(
        google_client
            .client_secret()
            .expect("desktop client secret")
            .expose_for_runtime(),
        "desktop-client-secret"
    );
    assert_eq!(
        google_client.authorization_endpoint(),
        "https://accounts.google.com/o/oauth2/auth"
    );
    assert_eq!(
        google_client.token_endpoint(),
        "https://oauth2.googleapis.com/token"
    );
    assert_eq!(google_client.redirect_uris(), ["http://localhost"]);
    assert_eq!(
        config.google_oauth_client_id(),
        Some("desktop-client-id.apps.googleusercontent.com")
    );
}

#[tokio::test]
async fn gmail_oauth_start_api_uses_configured_google_desktop_client_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping gmail oauth desktop config API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let vault_dir = tempdir().expect("vault tempdir");
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        AppConfig::from_pairs([
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
            (
                "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
            ),
        ])
        .expect("config"),
        database.clone(),
    );
    unlock_test_vault(app.clone()).await;

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/callback"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let authorization_url = body["authorization_url"]
        .as_str()
        .expect("authorization url");
    assert!(authorization_url.starts_with("https://accounts.google.com/o/oauth2/auth?"));
    assert!(authorization_url.contains("client_id=desktop-client-id.apps.googleusercontent.com"));
    assert!(authorization_url.contains("gmail.readonly"));
    assert!(authorization_url.contains("calendar.readonly"));
    assert!(authorization_url.contains("contacts.readonly"));

    drop(database);
}

#[tokio::test]
async fn gmail_oauth_start_api_requires_initialized_host_vault_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let app = build_router_with_database(
        AppConfig::from_pairs([
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
            (
                "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
            ),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/callback"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "host_vault_error");
    assert_eq!(body["message"], "host vault is not initialized");
}

#[tokio::test]
async fn gmail_oauth_start_api_reopens_initialized_host_vault_after_restart_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let initialized_app = build_router_with_database(
        AppConfig::from_pairs([
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
        .expect("config"),
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );
    unlock_test_vault(initialized_app).await;

    let restarted_app = build_router_with_database(
        AppConfig::from_pairs([
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
            (
                "HERMES_GOOGLE_OAUTH_CLIENT_CONFIG_JSON",
                r#"{
                    "installed": {
                        "client_id": "desktop-client-id.apps.googleusercontent.com",
                        "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                        "token_uri": "https://oauth2.googleapis.com/token",
                        "client_secret": "desktop-client-secret",
                        "redirect_uris": ["http://localhost"]
                    }
                }"#,
            ),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    );

    let response = restarted_app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/gmail/oauth/start",
            json!({
                "account_id": "gmail-primary",
                "display_name": "Google Workspace",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/callback"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let authorization_url = body["authorization_url"]
        .as_str()
        .expect("authorization url");
    assert!(authorization_url.starts_with("https://accounts.google.com/o/oauth2/auth?"));
    assert!(authorization_url.contains("client_id=desktop-client-id.apps.googleusercontent.com"));
}

#[tokio::test]
async fn gmail_oauth_callback_completes_pending_grant_without_api_secret() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let app = build_router_with_database(
        AppConfig::from_pairs([
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
        .expect("config"),
        database,
    );
    unlock_test_vault(app.clone()).await;

    let token_server = MockTokenServer::start();
    let suffix = unique_suffix();
    let account_id = format!("gmail-callback-{suffix}");
    let start_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/gmail/oauth/start",
            json!({
                "account_id": account_id,
                "display_name": "Google Workspace",
                "client_id": "desktop-client-id",
                "redirect_uri": "http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/callback",
                "app_return_url": "http://127.0.0.1:5174/?hermes_oauth=gmail_connected",
                "authorization_endpoint": format!("{}/authorize", token_server.base_url()),
                "token_endpoint": format!("{}/token", token_server.base_url())
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    let state = start_body["state"].as_str().expect("state");

    let callback_response = app
        .oneshot(get_request(&format!(
            "/api/v1/email-accounts/gmail/oauth/callback?code=authorization-code&state={state}"
        )))
        .await
        .expect("callback response");

    assert_eq!(callback_response.status(), StatusCode::OK);
    let callback_body = text_body(callback_response).await;
    assert!(callback_body.contains("Google mail connected"));
    assert!(callback_body.contains(&account_id));
    assert!(callback_body.contains("hermes:gmail-oauth-connected"));
    assert!(callback_body.contains("postMessage"));
    assert!(callback_body.contains("window.close"));
    assert!(callback_body.contains("hermes_oauth=gmail_connected"));
    assert!(!callback_body.contains("gmail-access-token"));
    assert!(!callback_body.contains("gmail-refresh-token"));

    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let account = communication_store
        .provider_account(&account_id)
        .await
        .expect("load provider account")
        .expect("provider account");
    assert_eq!(account.provider_kind, EmailProviderKind::Gmail);
    assert_eq!(account.external_account_id, account_id);
    assert!(account.config.get("access_token").is_none());
    assert!(account.config.get("refresh_token").is_none());

    let calendar_account_id = format!("google-calendar:{account_id}");
    let calendar_account = CalendarAccountStore::new(pool.clone())
        .get(&calendar_account_id)
        .await
        .expect("load calendar account")
        .expect("calendar account");
    assert_eq!(calendar_account.provider, "google");
    assert_eq!(calendar_account.account_name, "Google Workspace");
    assert_eq!(calendar_account.email.as_deref(), Some(account_id.as_str()));
    assert_eq!(
        calendar_account.credentials_reference.as_deref(),
        Some(format!("secret:provider-account:{account_id}:oauth_token").as_str())
    );
    assert_eq!(calendar_account.capabilities["mail_account_id"], account_id);
    assert_eq!(
        calendar_account.capabilities["connected_services"],
        json!(["calendar"])
    );

    let binding = communication_store
        .provider_account_secret_binding(&account_id, ProviderAccountSecretPurpose::OauthToken)
        .await
        .expect("load binding")
        .expect("secret binding");
    let secret_store = SecretReferenceStore::new(pool);
    let reference = secret_store
        .secret_reference(&binding.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference");
    assert_eq!(reference.store_kind, SecretStoreKind::HostVault);

    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock().expect("unlock host vault");
    let token_bundle = vault
        .resolve(&reference)
        .await
        .expect("resolve token bundle");
    let token_bundle: Value =
        serde_json::from_str(token_bundle.expose_for_runtime()).expect("token bundle json");
    assert_eq!(token_bundle["access_token"], "gmail-access-token");
    assert_eq!(token_bundle["refresh_token"], "gmail-refresh-token");
}

#[tokio::test]
async fn gmail_oauth_callback_rejects_unknown_state_without_api_secret() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(
                    "/api/v1/email-accounts/gmail/oauth/callback?code=authorization-code&state=oauth-state",
                )
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = text_body(response).await;
    assert!(body.contains("Google mail connection failed"));
    assert!(body.contains("expired"));
    assert!(!body.contains("authorization-code"));
}

#[tokio::test]
async fn gmail_oauth_callback_rejects_missing_code_without_leaking_secrets() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(get_request(
            "/api/v1/email-accounts/gmail/oauth/callback?state=oauth-state",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = text_body(response).await;
    assert!(body.contains("Google mail connection failed"));
    assert!(body.contains("authorization code"));
    assert!(!body.contains("access_token"));
    assert!(!body.contains("refresh_token"));
}

#[tokio::test]
async fn gmail_oauth_start_and_complete_still_require_api_secret() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config"),
        Database::disabled(),
    );

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/email-accounts/gmail/oauth/start")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "account_id": "gmail-primary",
                        "display_name": "Google Workspace",
                        "client_id": "desktop-client-id",
                        "redirect_uri": "http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/callback"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("start response");
    assert_eq!(start_response.status(), StatusCode::FORBIDDEN);

    let complete_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/email-accounts/gmail/oauth/complete")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "setup_id": "setup",
                        "state": "state",
                        "authorization_code": "code"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("complete response");
    assert_eq!(complete_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn gmail_oauth_setup_builds_pkce_url_and_persists_token_bundle_against_postgres() {
    let Some((database, communication_store, secret_store, suffix)) =
        live_setup_context("gmail oauth account setup").await
    else {
        return;
    };
    let token_server = MockTokenServer::start();
    let vault = DatabaseEncryptedSecretVault::new(
        database.pool().expect("configured pool").clone(),
        ResolvedSecret::new("gmail oauth vault key").expect("vault key"),
    );

    let service = EmailAccountSetupService::new(
        communication_store.clone(),
        secret_store.clone(),
        vault.clone(),
    );
    let pending = service
        .start_gmail_oauth(
            GmailOAuthSetupRequest::new(
                format!("acct_gmail_setup_{suffix}"),
                "Gmail setup",
                format!("gmail-setup-{suffix}@example.com"),
                "desktop-client-id",
                "http://127.0.0.1:18088/oauth/callback",
            )
            .authorization_endpoint(format!("{}/authorize", token_server.base_url()))
            .token_endpoint(format!("{}/token", token_server.base_url())),
        )
        .expect("start gmail oauth setup");

    assert!(pending.authorization_url.contains("code_challenge="));
    assert!(
        pending
            .authorization_url
            .contains("code_challenge_method=S256")
    );
    assert!(pending.authorization_url.contains("access_type=offline"));
    assert!(pending.authorization_url.contains("prompt=consent"));
    assert!(pending.authorization_url.contains("gmail.readonly"));
    assert!(!pending.authorization_url.contains(&pending.code_verifier));

    let completed = service
        .complete_gmail_oauth(pending.clone(), "authorization-code")
        .await
        .expect("complete gmail oauth setup");

    assert_eq!(completed.account_id, pending.account_id);
    assert_eq!(completed.secret_kind, SecretKind::OauthToken);
    assert_eq!(
        completed.store_kind,
        SecretStoreKind::DatabaseEncryptedVault
    );

    let account = communication_store
        .provider_account(&pending.account_id)
        .await
        .expect("load provider account")
        .expect("provider account exists");
    assert_eq!(account.provider_kind, EmailProviderKind::Gmail);
    assert_eq!(account.config["auth"], "oauth");
    assert_eq!(account.config["api"], "gmail");
    assert_eq!(account.config["oauth_client_id"], "desktop-client-id");
    assert!(account.config.get("access_token").is_none());
    assert!(account.config.get("refresh_token").is_none());

    let binding = communication_store
        .provider_account_secret_binding(
            &pending.account_id,
            ProviderAccountSecretPurpose::OauthToken,
        )
        .await
        .expect("load binding")
        .expect("binding exists");
    assert_eq!(binding.secret_ref, completed.secret_ref);

    let reference = secret_store
        .secret_reference(&completed.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference exists");
    assert_eq!(
        reference.store_kind,
        SecretStoreKind::DatabaseEncryptedVault
    );
    assert_eq!(reference.secret_kind, SecretKind::OauthToken);

    let token_bundle = vault
        .resolve(&reference)
        .await
        .expect("resolve token bundle");
    let token_bundle: Value =
        serde_json::from_str(token_bundle.expose_for_runtime()).expect("token bundle json");
    assert_eq!(token_bundle["access_token"], "gmail-access-token");
    assert_eq!(token_bundle["refresh_token"], "gmail-refresh-token");
    assert_eq!(token_bundle["client_id"], "desktop-client-id");

    let requests = token_server.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].body.contains("grant_type=authorization_code"));
    assert!(requests[0].body.contains("code=authorization-code"));
    assert!(requests[0].body.contains("code_verifier="));

    drop(database);
}

#[tokio::test]
async fn gmail_oauth_refresh_returns_runtime_access_token_and_updates_vault() {
    let Some((database, _communication_store, secret_store, suffix)) =
        live_setup_context("gmail oauth refresh").await
    else {
        return;
    };
    let token_server = MockTokenServer::start();
    let vault = DatabaseEncryptedSecretVault::new(
        database.pool().expect("configured pool").clone(),
        ResolvedSecret::new("refresh vault key").expect("vault key"),
    );
    let secret_ref = format!("secret:gmail:oauth:refresh-test:{suffix}");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::DatabaseEncryptedVault,
            "Gmail refresh credential",
        ))
        .await
        .expect("store refresh secret reference");
    vault
        .store_secret(
            &secret_ref,
            &json!({
                "token_url": format!("{}/token", token_server.base_url()),
                "client_id": "desktop-client-id",
                "access_token": "expired-access-token",
                "refresh_token": "gmail-refresh-token",
                "expires_at": "2000-01-01T00:00:00Z"
            })
            .to_string(),
        )
        .await
        .expect("store expired token bundle");

    let service = EmailAccountSetupService::new_for_vault_only(vault.clone());
    let access_token = service
        .refresh_gmail_access_token(&secret_ref)
        .await
        .expect("refresh gmail access token");

    assert_eq!(
        access_token.expose_for_runtime(),
        "gmail-refreshed-access-token"
    );

    let refreshed = vault
        .resolve(&secret_reference(&secret_ref))
        .await
        .expect("resolve refreshed token bundle");
    let refreshed: Value =
        serde_json::from_str(refreshed.expose_for_runtime()).expect("refreshed token bundle json");
    assert_eq!(refreshed["access_token"], "gmail-refreshed-access-token");
    assert_eq!(refreshed["refresh_token"], "gmail-refresh-token");

    let requests = token_server.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].body.contains("grant_type=refresh_token"));
    assert!(
        requests[0]
            .body
            .contains("refresh_token=gmail-refresh-token")
    );

    drop(database);
}

#[tokio::test]
async fn imap_account_setup_stores_encrypted_secret_in_database_against_postgres() {
    let Some((database, communication_store, secret_store, suffix)) =
        live_setup_context("imap account setup").await
    else {
        return;
    };
    let vault = DatabaseEncryptedSecretVault::new(
        database.pool().expect("configured pool").clone(),
        ResolvedSecret::new("imap vault key").expect("vault key"),
    );
    let service = EmailAccountSetupService::new(
        communication_store.clone(),
        secret_store.clone(),
        vault.clone(),
    );

    let account_id = format!("acct_imap_setup_{suffix}");
    let completed = service
        .setup_imap_account(
            ImapAccountSetupRequest::new(
                &account_id,
                EmailProviderKind::Icloud,
                "iCloud setup",
                format!("icloud-setup-{suffix}@icloud.com"),
                "imap.mail.me.com",
                993,
                true,
                "INBOX",
                format!("icloud-setup-{suffix}@icloud.com"),
                "icloud-app-password",
            )
            .secret_kind(SecretKind::AppPassword),
        )
        .await
        .expect("setup imap account");

    let account = communication_store
        .provider_account(&account_id)
        .await
        .expect("load provider account")
        .expect("provider account exists");
    assert_eq!(account.provider_kind, EmailProviderKind::Icloud);
    assert_eq!(account.config["host"], "imap.mail.me.com");
    assert_eq!(account.config["port"], 993);
    assert_eq!(account.config["tls"], true);
    assert_eq!(account.config["mailbox"], "INBOX");
    assert_eq!(
        account.config["username"],
        format!("icloud-setup-{suffix}@icloud.com")
    );
    assert!(account.config.get("password").is_none());
    assert!(account.config.get("app_password").is_none());

    let reference = secret_store
        .secret_reference(&completed.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference exists");
    assert_eq!(
        reference.store_kind,
        SecretStoreKind::DatabaseEncryptedVault
    );
    assert_eq!(reference.secret_kind, SecretKind::AppPassword);
    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve imap password")
            .expose_for_runtime(),
        "icloud-app-password"
    );

    drop(database);
}

#[tokio::test]
async fn icloud_account_setup_api_creates_calendar_account_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        AppConfig::from_pairs([
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
        .expect("config"),
        database.clone(),
    );
    unlock_test_vault(app.clone()).await;

    let account_id = "icloud-primary";
    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "icloud",
                "display_name": "Primary iCloud",
                "external_account_id": "user@icloud.com",
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "user@icloud.com",
                "password": "icloud-app-password",
                "secret_kind": "app_password"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], account_id);

    let pool = database.pool().expect("configured pool").clone();
    let account = CommunicationIngestionStore::new(pool.clone())
        .provider_account(account_id)
        .await
        .expect("load provider account")
        .expect("provider account");
    assert_eq!(account.provider_kind, EmailProviderKind::Icloud);
    assert_eq!(
        account.config["connected_services"],
        json!(["mail", "calendar", "contacts"])
    );
    assert_eq!(account.config["smtp_host"], "smtp.mail.me.com");
    assert_eq!(account.config["smtp_port"], 587);
    assert_eq!(account.config["smtp_tls"], true);
    assert_eq!(account.config["smtp_starttls"], true);
    assert_eq!(account.config["smtp_username"], "user@icloud.com");
    assert!(account.config.get("password").is_none());
    assert!(account.config.get("smtp_password").is_none());

    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool.clone());
    let imap_binding = communication_store
        .provider_account_secret_binding(account_id, ProviderAccountSecretPurpose::ImapPassword)
        .await
        .expect("load imap binding")
        .expect("imap binding");
    let smtp_binding = communication_store
        .provider_account_secret_binding(account_id, ProviderAccountSecretPurpose::SmtpPassword)
        .await
        .expect("load smtp binding")
        .expect("smtp binding");
    assert_eq!(
        imap_binding.secret_ref,
        "secret:provider-account:icloud-primary:imap_password"
    );
    assert_eq!(
        smtp_binding.secret_ref,
        "secret:provider-account:icloud-primary:smtp_password"
    );

    let smtp_reference = secret_store
        .secret_reference(&smtp_binding.secret_ref)
        .await
        .expect("load smtp secret reference")
        .expect("smtp secret reference");
    assert_eq!(smtp_reference.store_kind, SecretStoreKind::HostVault);
    assert_eq!(smtp_reference.secret_kind, SecretKind::AppPassword);
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    assert_eq!(
        vault
            .resolve(&smtp_reference)
            .await
            .expect("resolve smtp password")
            .expose_for_runtime(),
        "icloud-app-password"
    );

    let calendar_account_id = format!("icloud-calendar:{account_id}");
    let calendar_account = CalendarAccountStore::new(pool)
        .get(&calendar_account_id)
        .await
        .expect("load calendar account")
        .expect("calendar account");
    assert_eq!(calendar_account.provider, "apple");
    assert_eq!(calendar_account.account_name, "Primary iCloud");
    assert_eq!(calendar_account.email.as_deref(), Some("user@icloud.com"));
    assert_eq!(
        calendar_account.credentials_reference.as_deref(),
        Some("secret:provider-account:icloud-primary:imap_password")
    );
    assert_eq!(calendar_account.capabilities["mail_account_id"], account_id);
    assert_eq!(calendar_account.capabilities["source_provider"], "icloud");
    assert_eq!(
        calendar_account.capabilities["connected_services"],
        json!(["calendar"])
    );
}

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
    let send_body: Value = serde_json::from_slice(
        &to_bytes(send_response.into_body(), 1024 * 1024)
            .await
            .expect("read send body"),
    )
    .expect("send json body");
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
    let body: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read body"),
    )
    .expect("json body");
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
    let body: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read body"),
    )
    .expect("json body");
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

#[tokio::test]
async fn startup_reconciles_icloud_account_from_host_vault_manifest_after_postgres_metadata_wipe() {
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
    let app = build_router_with_database(config.clone(), database.clone());
    unlock_test_vault(app.clone()).await;

    let account_id = "icloud-recover";
    let secret_ref = "secret:provider-account:icloud-recover:imap_password";
    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "icloud",
                "display_name": "Recovered iCloud",
                "external_account_id": "recover@icloud.com",
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "recover@icloud.com",
                "password": "icloud-app-password",
                "secret_kind": "app_password"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let pool = database.pool().expect("configured pool").clone();
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .upsert_account_secret_manifest_entry(
            secret_ref,
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id,
                purpose: ProviderAccountSecretPurpose::ImapPassword.as_str(),
                secret_kind: SecretKind::AppPassword.as_str(),
                label: "IMAP password",
                metadata: &json!({
                    "provider": "icloud",
                    "account_id": account_id
                }),
            },
        )
        .expect("write sparse manifest entry");

    let _enriching_app = build_router_with_database(config.clone(), database.clone());
    wait_for_manifest_metadata_key(&vault, secret_ref, "display_name").await;

    sqlx::query("DELETE FROM calendar_accounts WHERE account_id = $1")
        .bind(format!("icloud-calendar:{account_id}"))
        .execute(&pool)
        .await
        .expect("delete calendar metadata");
    sqlx::query("DELETE FROM communication_provider_account_secret_refs WHERE account_id = $1")
        .bind(account_id)
        .execute(&pool)
        .await
        .expect("delete secret binding");
    sqlx::query("DELETE FROM communication_provider_accounts WHERE account_id = $1")
        .bind(account_id)
        .execute(&pool)
        .await
        .expect("delete provider account");
    sqlx::query("DELETE FROM secret_references WHERE secret_ref = $1")
        .bind(secret_ref)
        .execute(&pool)
        .await
        .expect("delete secret reference");

    assert!(
        CommunicationIngestionStore::new(pool.clone())
            .provider_account(account_id)
            .await
            .expect("load deleted account")
            .is_none()
    );

    let restarted_database = Database::connect(Some(&database_url))
        .await
        .expect("restarted database connection");
    let _restarted_app = build_router_with_database(config, restarted_database.clone());
    let restarted_pool = restarted_database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(restarted_pool.clone());
    let secret_store = SecretReferenceStore::new(restarted_pool.clone());

    let account = wait_for_provider_account(&communication_store, account_id).await;
    assert_eq!(account.provider_kind, EmailProviderKind::Icloud);
    assert_eq!(account.display_name, "Recovered iCloud");
    assert_eq!(account.external_account_id, "recover@icloud.com");
    assert_eq!(
        account.config["connected_services"],
        json!(["mail", "calendar", "contacts"])
    );

    let reference = wait_for_secret_reference(&secret_store, secret_ref).await;
    assert_eq!(reference.store_kind, SecretStoreKind::HostVault);
    assert_eq!(reference.secret_kind, SecretKind::AppPassword);

    let binding = wait_for_provider_account_secret_binding(
        &communication_store,
        account_id,
        ProviderAccountSecretPurpose::ImapPassword,
    )
    .await;
    assert_eq!(binding.secret_ref, secret_ref);

    let calendar_store = CalendarAccountStore::new(restarted_pool.clone());
    let calendar_account =
        wait_for_calendar_account(&calendar_store, &format!("icloud-calendar:{account_id}")).await;
    assert_eq!(calendar_account.provider, "apple");
    assert_eq!(
        calendar_account.email.as_deref(),
        Some("recover@icloud.com")
    );
    assert_eq!(
        calendar_account.credentials_reference.as_deref(),
        Some(secret_ref)
    );

    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored secret")
            .expose_for_runtime(),
        "icloud-app-password"
    );
}

#[tokio::test]
async fn imap_account_setup_api_requires_configured_database() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)]).expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/imap",
            json!({
                "account_id": "acct_no_vault",
                "provider_kind": "imap",
                "display_name": "No vault",
                "external_account_id": "no-vault@example.net",
                "host": "imap.example.net",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "no-vault@example.net",
                "password": "secret"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "database_not_configured");
}

#[tokio::test]
async fn imap_account_setup_api_requires_initialized_host_vault_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live account setup missing host vault test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let vault_dir = tempdir().expect("vault tempdir");
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        AppConfig::from_pairs([
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
        ])
        .expect("config"),
        database.clone(),
    );

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/imap",
            json!({
                "account_id": "acct_no_vault_key",
                "provider_kind": "imap",
                "display_name": "No vault key",
                "external_account_id": "no-vault-key@example.net",
                "host": "imap.example.net",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "no-vault-key@example.net",
                "password": "secret"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "host_vault_error");
    assert_eq!(body["message"], "host vault is not initialized");
}

#[derive(Clone, Debug)]
struct TokenRequest {
    body: String,
}

struct MockTokenServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<TokenRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockTokenServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind token server");
        let addr = listener.local_addr().expect("token server addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let request = read_http_request(&mut stream);
                if request.body.is_empty() {
                    break;
                }
                let body = if request.body.contains("grant_type=refresh_token") {
                    json!({
                        "access_token": "gmail-refreshed-access-token",
                        "expires_in": 3600,
                        "token_type": "Bearer"
                    })
                    .to_string()
                } else {
                    json!({
                        "access_token": "gmail-access-token",
                        "refresh_token": "gmail-refresh-token",
                        "expires_in": 3600,
                        "token_type": "Bearer",
                        "scope": "https://www.googleapis.com/auth/gmail.readonly"
                    })
                    .to_string()
                };
                requests_for_thread
                    .lock()
                    .expect("requests lock")
                    .push(request);
                write_http_response(&mut stream, &body);
            }
        });

        Self {
            addr,
            requests,
            handle: Some(handle),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn requests(&self) -> Vec<TokenRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

impl Drop for MockTokenServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("token server join");
        }
    }
}

struct MockSmtpServer {
    addr: SocketAddr,
    commands: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockSmtpServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind SMTP server");
        let addr = listener.local_addr().expect("SMTP server addr");
        let commands = Arc::new(Mutex::new(Vec::new()));
        let commands_for_thread = Arc::clone(&commands);
        let handle = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .expect("set SMTP read timeout");
            write!(stream, "220 mock.smtp.local ESMTP\r\n").expect("write greeting");

            let mut reader = BufReader::new(stream.try_clone().expect("clone SMTP stream"));
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => break,
                    Err(error) => panic!("read SMTP line: {error}"),
                }
                let command = line.trim_end().to_owned();
                commands_for_thread
                    .lock()
                    .expect("SMTP commands lock")
                    .push(command.clone());
                if command.starts_with("EHLO") {
                    write!(stream, "250-mock.smtp.local\r\n250 AUTH LOGIN\r\n")
                        .expect("write EHLO response");
                } else if command == "AUTH LOGIN" {
                    write!(stream, "334 VXNlcm5hbWU6\r\n").expect("write username prompt");
                } else if command == "c2VuZGVyQGV4YW1wbGUuY29t" {
                    write!(stream, "334 UGFzc3dvcmQ6\r\n").expect("write password prompt");
                } else if command == "c210cC1hcHAtcGFzc3dvcmQ=" {
                    write!(stream, "235 Authentication successful\r\n").expect("write auth ok");
                } else if command.starts_with("MAIL FROM") || command.starts_with("RCPT TO") {
                    write!(stream, "250 OK\r\n").expect("write envelope ok");
                } else if command == "DATA" {
                    write!(stream, "354 End data with <CR><LF>.<CR><LF>\r\n")
                        .expect("write DATA response");
                    loop {
                        let mut data_line = String::new();
                        if reader
                            .read_line(&mut data_line)
                            .expect("read SMTP data line")
                            == 0
                        {
                            return;
                        }
                        let data_line = data_line.trim_end().to_owned();
                        commands_for_thread
                            .lock()
                            .expect("SMTP commands lock")
                            .push(data_line.clone());
                        if data_line == "." {
                            break;
                        }
                    }
                    write!(stream, "250 mock-message-id queued\r\n").expect("write queued");
                } else if command == "QUIT" {
                    write!(stream, "221 Bye\r\n").expect("write bye");
                    break;
                } else {
                    write!(stream, "250 OK\r\n").expect("write default ok");
                }
            }
        });

        Self {
            addr,
            commands,
            handle: Some(handle),
        }
    }

    fn addr(&self) -> SocketAddr {
        self.addr
    }

    fn commands(&self) -> Vec<String> {
        self.commands.lock().expect("SMTP commands lock").clone()
    }
}

impl Drop for MockSmtpServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("SMTP server join");
        }
    }
}

fn read_http_request(stream: &mut TcpStream) -> TokenRequest {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream);
    let mut content_length = 0usize;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read request line");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().expect("content length");
        }
    }

    let mut body = vec![0_u8; content_length];
    use std::io::Read;
    reader.read_exact(&mut body).expect("read request body");

    TokenRequest {
        body: String::from_utf8(body).expect("utf8 body"),
    }
}

fn write_http_response(stream: &mut TcpStream, body: &str) {
    let result = write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    if let Err(error) = result {
        assert_eq!(
            error.kind(),
            ErrorKind::BrokenPipe,
            "write response: {error}"
        );
    }
}

async fn live_setup_context(
    test_name: &str,
) -> Option<(
    Database,
    CommunicationIngestionStore,
    SecretReferenceStore,
    u128,
)> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);

    Some((database, communication_store, secret_store, unique_suffix()))
}

fn secret_reference(secret_ref: &str) -> hermes_hub_backend::platform::secrets::SecretReference {
    let now = chrono::Utc::now();

    hermes_hub_backend::platform::secrets::SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind: SecretKind::OauthToken,
        store_kind: SecretStoreKind::DatabaseEncryptedVault,
        label: "Gmail OAuth".to_owned(),
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    }
}

fn json_request_with_token_and_actor(
    uri: &str,
    body: Value,
    token: &str,
    _actor_id: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

async fn unlock_test_vault<S>(app: S)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let entropy_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);

    let create_response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/vault/create",
            json!({}),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);
}

async fn wait_for_provider_account(
    communication_store: &CommunicationIngestionStore,
    account_id: &str,
) -> hermes_hub_backend::domains::mail::core::ProviderAccount {
    for _ in 0..50 {
        if let Some(account) = communication_store
            .provider_account(account_id)
            .await
            .expect("load provider account")
        {
            return account;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("provider account {account_id} was not reconciled");
}

async fn wait_for_secret_reference(
    secret_store: &SecretReferenceStore,
    secret_ref: &str,
) -> hermes_hub_backend::platform::secrets::SecretReference {
    for _ in 0..50 {
        if let Some(reference) = secret_store
            .secret_reference(secret_ref)
            .await
            .expect("load secret reference")
        {
            return reference;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("secret reference {secret_ref} was not reconciled");
}

async fn wait_for_provider_account_secret_binding(
    communication_store: &CommunicationIngestionStore,
    account_id: &str,
    secret_purpose: ProviderAccountSecretPurpose,
) -> hermes_hub_backend::domains::mail::core::ProviderAccountSecretBinding {
    for _ in 0..50 {
        if let Some(binding) = communication_store
            .provider_account_secret_binding(account_id, secret_purpose)
            .await
            .expect("load provider account secret binding")
        {
            return binding;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("provider account secret binding {account_id}/{secret_purpose:?} was not reconciled");
}

async fn wait_for_calendar_account(
    calendar_store: &CalendarAccountStore,
    account_id: &str,
) -> hermes_hub_backend::domains::calendar::events::CalendarAccount {
    for _ in 0..50 {
        if let Some(account) = calendar_store
            .get(account_id)
            .await
            .expect("load calendar account")
        {
            return account;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("calendar account {account_id} was not reconciled");
}

async fn wait_for_manifest_metadata_key(vault: &HostVault, secret_ref: &str, key: &str) {
    for _ in 0..50 {
        let has_key = vault
            .account_secret_manifest()
            .expect("read host vault manifest")
            .into_iter()
            .any(|entry| entry.secret_ref == secret_ref && entry.metadata.get(key).is_some());
        if has_key {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("manifest entry {secret_ref} was not enriched with {key}");
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
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

async fn text_body(response: axum::response::Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    String::from_utf8(body.to_vec()).expect("utf8 body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
