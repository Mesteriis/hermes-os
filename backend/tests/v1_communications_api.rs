use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

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
