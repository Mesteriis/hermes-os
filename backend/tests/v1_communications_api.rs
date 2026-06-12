use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-test-token";

fn cfg() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", T)]).expect("cfg")
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", T)
        .body(Body::empty())
        .expect("req")
}

fn pget(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

fn pget_with_actor(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", T)
        .header("x-hermes-actor-id", "hermes-frontend")
        .body(Body::from(body.to_string()))
        .expect("req")
}

fn pput(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

fn del(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-hermes-secret", T)
        .body(Body::empty())
        .expect("req")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

async fn router(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", T), ("DATABASE_URL", db)]).expect("cfg"),
        database,
    )
}

macro_rules! v1_read_test {
    ($name:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip {}: no DB", stringify!($name));
                return;
            };
            let r = router(&db).await;
            let resp = r.oneshot(get($path)).await.expect("r");
            assert!(
                !resp.status().is_server_error(),
                "{}: status={}",
                stringify!($name),
                resp.status()
            );
        }
    };
}

v1_read_test!(v1_messages_list, "/api/v1/communications/messages");
v1_read_test!(v1_message_states, "/api/v1/communications/messages/states");
v1_read_test!(v1_threads_list, "/api/v1/communications/threads");
v1_read_test!(
    v1_thread_messages,
    "/api/v1/communications/threads/messages?thread_id=thread%3Atest"
);
v1_read_test!(v1_search, "/api/v1/communications/search?q=test");
v1_read_test!(v1_personas_list, "/api/v1/communications/personas");
v1_read_test!(v1_drafts_list, "/api/v1/communications/drafts");
v1_read_test!(v1_invoices_list, "/api/v1/communications/finance/invoices");
v1_read_test!(
    v1_analytics_health,
    "/api/v1/communications/analytics/health"
);
v1_read_test!(
    v1_analytics_senders,
    "/api/v1/communications/analytics/senders"
);
v1_read_test!(v1_subscriptions, "/api/v1/communications/subscriptions");
v1_read_test!(
    v1_dup_attachments,
    "/api/v1/communications/attachments/duplicates"
);
v1_read_test!(v1_legal_list, "/api/v1/communications/legal");
v1_read_test!(v1_certs_list, "/api/v1/communications/certificates");
v1_read_test!(
    v1_certs_expiring,
    "/api/v1/communications/certificates/expiring"
);
v1_read_test!(v1_rich_templates, "/api/v1/communications/templates/rich");
v1_read_test!(v1_blockers, "/api/v1/communications/blockers");
v1_read_test!(v1_sync_status, "/api/v1/email-accounts/sync-status");

// ── Write-like endpoints (may fail gracefully without data) ────────────────

macro_rules! v1_post_test {
    ($name:ident, $path:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip {}: no DB", stringify!($name));
                return;
            };
            let r = router(&db).await;
            let resp = r.oneshot(pget($path, $body)).await.expect("r");
            assert!(
                resp.status().is_success()
                    || resp.status() == StatusCode::NOT_FOUND
                    || resp.status() == StatusCode::BAD_REQUEST
                    || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
                "{}: status={}",
                stringify!($name),
                resp.status()
            );
        }
    };
}

v1_post_test!(
    v1_pin_msg,
    "/api/v1/communications/messages/msg:fake/pin",
    json!({})
);
v1_post_test!(
    v1_snooze_msg,
    "/api/v1/communications/messages/msg:fake/snooze",
    json!({"until": "2027-01-01T00:00:00Z"})
);
v1_post_test!(
    v1_mute_msg,
    "/api/v1/communications/messages/msg:fake/mute",
    json!({})
);
v1_post_test!(
    v1_label_msg,
    "/api/v1/communications/messages/msg:fake/labels",
    json!({"label": "important"})
);
v1_post_test!(
    v1_render_tpl,
    "/api/v1/communications/templates/rich/render",
    json!({"template": "Hello {{name}}", "context": {"name": "Test"}})
);

#[tokio::test]
async fn workflow_action_endpoint_exists_without_database() {
    let app = build_router(cfg());
    let response = app
        .oneshot(pget_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": "workflow-action-no-db",
                "action": "reply",
                "source": { "kind": "communication_message", "id": "msg:no-db" }
            }),
        ))
        .await
        .expect("workflow action no-db response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn v1_sync_settings_default_update_and_manual_sync_status_against_postgres() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip sync settings: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("db");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let account_id = format!("acct-sync-api-{suffix}");
    CommunicationIngestionStore::new(pool)
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Sync API IMAP",
            format!("sync-api-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let r = router(&db).await;
    let settings_path = format!("/api/v1/email-accounts/{account_id}/sync-settings");
    let resp = r
        .clone()
        .oneshot(get(&settings_path))
        .await
        .expect("get settings");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read settings"),
    )
    .expect("settings json");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["sync_enabled"], true);
    assert_eq!(body["batch_size"], 100);
    assert_eq!(body["poll_interval_seconds"], 300);

    let resp = r
        .clone()
        .oneshot(pput(
            &settings_path,
            json!({"sync_enabled": false, "batch_size": 7, "poll_interval_seconds": 600}),
        ))
        .await
        .expect("put settings");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read updated settings"),
    )
    .expect("updated settings json");
    assert_eq!(body["sync_enabled"], false);
    assert_eq!(body["batch_size"], 7);
    assert_eq!(body["poll_interval_seconds"], 600);

    let resp = r
        .clone()
        .oneshot(pget(
            &format!("/api/v1/email-accounts/{account_id}/sync-now"),
            json!({}),
        ))
        .await
        .expect("sync now");
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "sync-now should return structured result or safe configuration error, got {}",
        resp.status()
    );
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read sync-now"),
    )
    .expect("sync-now json");
    assert_eq!(body["account_id"], account_id);
    assert!(body.get("status").is_some());
    assert!(body.get("phase").is_some());

    let resp = r
        .oneshot(pget(
            &format!("/api/v1/email-accounts/{account_id}/sync-full-resync"),
            json!({}),
        ))
        .await
        .expect("sync full resync");
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "sync-full-resync should return structured result or safe configuration error, got {}",
        resp.status()
    );
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read sync-full-resync"),
    )
    .expect("sync-full-resync json");
    assert_eq!(body["account_id"], account_id);
    assert!(body.get("status").is_some());
    assert!(body.get("phase").is_some());
}

#[tokio::test]
async fn v1_send_requires_explicit_provider_write_confirmation() {
    let ctx = TestContext::new().await;
    let r = router(&ctx.connection_string()).await;
    let resp = r
        .oneshot(pget_with_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": "icloud-primary",
                "to": ["recipient@example.com"],
                "subject": "Provider write guard",
                "body_text": "This request must not send without confirmation."
            }),
        ))
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("json response");
    assert_eq!(body["error"], "provider_write_confirmation_required");
}

// ── Auth rejection ─────────────────────────────────────────────────────────

#[tokio::test]
async fn v1_communications_reject_no_secret() {
    let r = build_router(cfg());
    let resp = r
        .oneshot(
            Request::builder()
                .uri("/api/v1/communications/messages")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("r");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── Specific message endpoints (fail gracefully) ───────────────────────────

#[tokio::test]
async fn v1_message_explain_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(get(
            "/api/v1/communications/messages/msg:nonexistent/explain",
        ))
        .await
        .expect("r");
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

#[tokio::test]
async fn v1_message_smart_cc_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(get(
            "/api/v1/communications/messages/msg:nonexistent/smart-cc",
        ))
        .await
        .expect("r");
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

#[tokio::test]
async fn v1_message_export_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(get(
            "/api/v1/communications/messages/msg:nonexistent/export",
        ))
        .await
        .expect("r");
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

#[tokio::test]
async fn v1_spf_dkim_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(get(
            "/api/v1/communications/messages/msg:nonexistent/spf-dkim",
        ))
        .await
        .expect("r");
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

#[tokio::test]
async fn v1_detect_lang_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(get(
            "/api/v1/communications/messages/msg:nonexistent/detect-language",
        ))
        .await
        .expect("r");
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

#[tokio::test]
async fn v1_signature_check_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(get(
            "/api/v1/communications/messages/msg:nonexistent/signature",
        ))
        .await
        .expect("r");
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}

// ── Additional V1 communications handlers ──────────────────────────────────

// Persona POST
#[tokio::test]
async fn v1_post_persona() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(pget(
            "/api/v1/communications/personas",
            json!({"name": "Test Persona", "context": "work", "email": "test@example.com"}),
        ))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "persona post={}",
        resp.status()
    );
}

// Draft POST
#[tokio::test]
async fn v1_post_draft() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(pget(
            "/api/v1/communications/drafts",
            json!({"subject": "Test Draft", "body": "Draft body"}),
        ))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "draft post={}",
        resp.status()
    );
}

// Invoice POST
#[tokio::test]
async fn v1_post_invoice() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(pget(
            "/api/v1/communications/finance/invoices",
            json!({"vendor": "Test Vendor", "amount": "100.00", "currency": "EUR"}),
        ))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "invoice post={}",
        resp.status()
    );
}

// Legal POST
#[tokio::test]
async fn v1_post_legal() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(pget(
            "/api/v1/communications/legal",
            json!({"doc_type": "nda", "title": "Test NDA"}),
        ))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "legal post={}",
        resp.status()
    );
}

// Cert POST
#[tokio::test]
async fn v1_post_cert() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(pget(
            "/api/v1/communications/certificates",
            json!({"domain": "example.com", "cert_data": "test-cert-data"}),
        ))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "cert post={}",
        resp.status()
    );
}

// Rich template POST
#[tokio::test]
async fn v1_post_rich_template() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(pget(
            "/api/v1/communications/templates/rich",
            json!({"name": "Test Template", "content": "Hello {{name}}"}),
        ))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "template post={}",
        resp.status()
    );
}

// Send / Reply / Forward (will fail gracefully without real provider)
macro_rules! v1_msg_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip");
                return;
            };
            let r = router(&db).await;
            let resp = r
                .oneshot(pget(
                    &format!("/api/v1/communications/messages/msg:fake/{}", $path_suffix),
                    $body,
                ))
                .await
                .expect("r");
            assert!(
                !resp.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                resp.status()
            );
        }
    };
}

v1_msg_post_test!(
    v1_send,
    "send",
    json!({"to": "test@example.com", "subject": "Test", "body": "Hello"})
);
v1_msg_post_test!(v1_reply, "reply", json!({"body": "Reply text"}));
v1_msg_post_test!(v1_reply_all, "reply-all", json!({"body": "Reply all text"}));
v1_msg_post_test!(v1_forward, "forward", json!({"to": "fwd@example.com"}));
v1_msg_post_test!(
    v1_forward_eml,
    "forward-eml",
    json!({"to": "fwd@example.com"})
);
v1_msg_post_test!(v1_imap_mark_read, "imap-mark-read", json!({}));
v1_msg_post_test!(v1_imap_delete, "imap-delete", json!({}));
v1_msg_post_test!(v1_translate, "translate", json!({"target_language": "es"}));
v1_msg_post_test!(v1_ai_reply, "ai-reply", json!({"prompt": "Reply to this"}));
v1_msg_post_test!(
    v1_ai_reply_variants,
    "ai-reply-variants",
    json!({"prompt": "Reply variants"})
);
v1_msg_post_test!(v1_extract_tasks, "extract-tasks", json!({}));
v1_msg_post_test!(v1_extract_notes, "extract-notes", json!({}));
v1_msg_post_test!(
    v1_message_analyze,
    "analyze",
    json!({"analysis_type": "sentiment"})
);

// Additional GET endpoints

#[tokio::test]
async fn v1_message_detail_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(get("/api/v1/communications/messages/msg:fake"))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "msg detail={}",
        resp.status()
    );
}

#[tokio::test]
async fn v1_draft_detail_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let s = uid();
    let r = router(&db).await;
    let resp = r
        .oneshot(get(&format!(
            "/api/v1/communications/drafts/draft:fake-{s}"
        )))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "draft detail={}",
        resp.status()
    );
}

// Delete endpoints
#[tokio::test]
async fn v1_delete_draft() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let s = uid();
    let r = router(&db).await;
    let resp = r
        .oneshot(del(&format!(
            "/api/v1/communications/drafts/draft:fake-{s}"
        )))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "delete draft={}",
        resp.status()
    );
}

#[tokio::test]
async fn v1_delete_message_label() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(del("/api/v1/communications/messages/msg:fake/labels"))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "delete label={}",
        resp.status()
    );
}

#[tokio::test]
async fn v1_imap_delete_alias_moves_message_to_local_trash_against_postgres() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip imap delete local trash: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("db");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let account_id = format!("acct-local-trash-api-{suffix}");
    let message_id = seed_projected_message(
        pool,
        &account_id,
        &format!("provider-local-trash-api-{suffix}"),
        "Local trash API",
    )
    .await;

    let r = router(&db).await;
    let resp = r
        .clone()
        .oneshot(pget(
            &format!("/api/v1/communications/messages/{message_id}/imap-delete"),
            json!({}),
        ))
        .await
        .expect("imap delete alias");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read imap-delete body"),
    )
    .expect("imap-delete json");
    assert_eq!(body["deleted"], true);
    assert_eq!(body["local_state"], "trash");

    let resp = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&q=Local%20trash%20API"
        )))
        .await
        .expect("active list");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read active list"),
    )
    .expect("active list json");
    assert_eq!(body["items"].as_array().expect("items").len(), 0);

    let resp = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&q=Local%20trash%20API&local_state=trash"
        )))
        .await
        .expect("trash list");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read trash list"),
    )
    .expect("trash list json");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["items"][0]["local_state"], "trash");

    let resp = r
        .oneshot(pget(
            &format!("/api/v1/communications/messages/{message_id}/restore"),
            json!({}),
        ))
        .await
        .expect("restore local trash");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read restore body"),
    )
    .expect("restore json");
    assert_eq!(body["local_state"], "active");
}

// Workflow state PUT
#[tokio::test]
async fn v1_put_workflow_state() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(pput(
            "/api/v1/communications/messages/msg:fake/workflow-state",
            json!({"state": "reviewed"}),
        ))
        .await
        .expect("r");
    assert!(
        !resp.status().is_server_error(),
        "workflow state={}",
        resp.status()
    );
}

#[tokio::test]
async fn workflow_action_create_task_is_idempotent_and_records_safe_event() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip workflow action task: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("db");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-action-{suffix}"),
        &format!("provider-workflow-action-{suffix}"),
        &format!("Workflow action task {suffix}"),
    )
    .await;
    let r = router(&db).await;
    let command_id = format!("workflow-action-task-{suffix}");
    let body = json!({
        "command_id": command_id,
        "action": "create_task",
        "source": { "kind": "communication_message", "id": message_id },
        "input": { "title": "Confirm integration access" }
    });

    let first = r
        .clone()
        .oneshot(pget_with_actor("/api/v1/workflow-actions", body.clone()))
        .await
        .expect("first workflow action response");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    assert_eq!(
        first_body["event_id"],
        json!(format!("workflow_action:{command_id}"))
    );
    assert_eq!(first_body["target"]["kind"], "task");
    assert_eq!(first_body["provenance"]["source_id"], message_id);

    let second = r
        .oneshot(pget_with_actor("/api/v1/workflow-actions", body))
        .await
        .expect("second workflow action response");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    assert_eq!(second_body["target"], first_body["target"]);

    let task_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM tasks WHERE source_id = $1 AND source_type = 'message'",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("task count");
    assert_eq!(task_count, 1);

    let event_payload: Value =
        sqlx::query_scalar("SELECT payload FROM event_log WHERE event_id = $1")
            .bind(format!("workflow_action:{command_id}"))
            .fetch_one(&pool)
            .await
            .expect("workflow event payload");
    assert!(
        !event_payload
            .to_string()
            .contains("Body for local trash API")
    );
}

#[tokio::test]
async fn workflow_action_create_note_creates_markdown_document() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip workflow action note: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("db");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let r = router(&db).await;
    let command_id = format!("workflow-action-note-{suffix}");

    let resp = r
        .oneshot(pget_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": command_id,
                "action": "create_note",
                "input": {
                    "title": "Follow-up note",
                    "body": "Remember to verify keys with the integration owner."
                }
            }),
        ))
        .await
        .expect("workflow note response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert_eq!(body["target"]["kind"], "document");
    let document_id = body["target"]["id"].as_str().expect("document id");
    let document_kind: String =
        sqlx::query_scalar("SELECT document_kind FROM documents WHERE document_id = $1")
            .bind(document_id)
            .fetch_one(&pool)
            .await
            .expect("document kind");
    assert_eq!(document_kind, "markdown");
}

#[tokio::test]
async fn workflow_action_create_event_requires_start_and_end() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip workflow action event validation: no DB");
        return;
    };
    let r = router(&db).await;
    let resp = r
        .oneshot(pget_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": format!("workflow-action-event-missing-{}", uid()),
                "action": "create_event",
                "input": { "title": "Missing time event" }
            }),
        ))
        .await
        .expect("workflow event validation response");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn workflow_action_archive_transitions_message_locally() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip workflow action archive: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("db");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-archive-{suffix}"),
        &format!("provider-workflow-archive-{suffix}"),
        &format!("Workflow archive {suffix}"),
    )
    .await;
    let r = router(&db).await;

    let resp = r
        .oneshot(pget_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": format!("workflow-action-archive-{suffix}"),
                "action": "archive",
                "source": { "kind": "communication_message", "id": message_id }
            }),
        ))
        .await
        .expect("workflow archive response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert_eq!(body["status"], "archived");
    let workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("workflow state");
    assert_eq!(workflow_state, "archived");
}

async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Seed Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            account_id,
            "email_message",
            provider_record_id,
            format!("sha256:{provider_record_id}"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": subject,
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Body for local trash API"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
