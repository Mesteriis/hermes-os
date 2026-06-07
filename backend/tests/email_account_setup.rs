use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::mail::accounts::{
    EmailAccountSetupService, GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, ProviderAccountSecretPurpose,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::secrets::DatabaseEncryptedSecretVault;
use hermes_hub_backend::platform::secrets::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReferenceStore, SecretResolver,
    SecretStoreKind,
};
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "account-setup-test-token";
const LOCAL_API_ACTOR_ID: &str = "account-setup-test-client";
const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";

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
async fn imap_account_setup_api_requires_configured_database() {
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN)]).expect("config"),
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
            LOCAL_API_ACTOR_ID,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "database_not_configured");
}

#[tokio::test]
async fn imap_account_setup_api_requires_configured_database_vault_key_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live account setup missing vault key test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN)]).expect("config"),
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
            LOCAL_API_ACTOR_ID,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "secret_vault_not_configured");
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
        if let Some(handle) = self.handle.take() {
            handle.join().expect("token server join");
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
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().expect("content length");
            }
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
    write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    )
    .expect("write response");
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
    actor_id: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, actor_id)
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
