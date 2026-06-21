use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
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
v1_read_test!(
    v1_sync_status,
    "/api/v1/integrations/mail/accounts/sync-status"
);

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
async fn v1_sync_settings_default_update_and_manual_sync_status_against_postgres() {
    let context = TestContext::new().await;
    let db = context.connection_string();
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-sync-api-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Sync API IMAP",
            format!("sync-api-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let r = router(&db).await;
    let settings_path = format!("/api/v1/integrations/mail/accounts/{account_id}/sync-settings");
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
            &format!("/api/v1/integrations/mail/accounts/{account_id}/sync-now"),
            json!({}),
        ))
        .await
        .expect("sync now");
    let sync_now_status = resp.status();
    assert!(
        sync_now_status == StatusCode::OK || sync_now_status == StatusCode::BAD_REQUEST,
        "sync-now should return structured result or safe configuration error, got {sync_now_status:?}",
    );
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read sync-now"),
    )
    .expect("sync-now json");
    if sync_now_status == StatusCode::BAD_REQUEST {
        assert_eq!(body["error"], "invalid_communication_query");
        return;
    }
    assert_eq!(body["account_id"], account_id);
    assert!(body.get("status").is_some());
    assert!(body.get("phase").is_some());
    let run_id = body["run_id"].as_str().expect("sync run id");
    let sync_events = sqlx::query(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'kind' = 'mail_sync_run'
          AND subject->>'id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(&pool)
    .await
    .expect("sync events");
    let sync_event_types = sync_events
        .iter()
        .map(|row| row.get::<String, _>("event_type"))
        .collect::<Vec<_>>();
    assert_eq!(
        sync_event_types,
        vec!["mail.sync.started", "mail.sync.skipped"]
    );
    let skipped_payload = sync_events
        .last()
        .expect("skipped event")
        .get::<Value, _>("payload");
    assert_eq!(skipped_payload["account_id"], account_id);
    assert_eq!(skipped_payload["run_id"], run_id);
    assert_eq!(skipped_payload["status"], "skipped");
    let sync_observation_kinds: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT kind.code AS kind_code
        FROM observation_links link
        JOIN observations observation ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'mail_sync_run'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(&pool)
    .await
    .expect("sync observations");
    assert_eq!(
        sync_observation_kinds,
        vec![
            "COMMUNICATION_MAIL_SYNC_RUN".to_owned(),
            "COMMUNICATION_MAIL_SYNC_RUN_STATUS".to_owned(),
        ]
    );
    let skipped_relationship_kind: String = sqlx::query_scalar(
        r#"
        SELECT relationship_kind
        FROM observation_links
        WHERE domain = 'communications'
          AND entity_kind = 'mail_sync_run'
          AND entity_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(run_id)
    .fetch_one(&pool)
    .await
    .expect("skipped sync relationship kind");
    assert_eq!(skipped_relationship_kind, "skipped");

    let resp = r
        .oneshot(pget(
            &format!("/api/v1/integrations/mail/accounts/{account_id}/sync-full-resync"),
            json!({}),
        ))
        .await
        .expect("sync full resync");
    let full_resync_status = resp.status();
    assert!(
        full_resync_status == StatusCode::OK || full_resync_status == StatusCode::BAD_REQUEST,
        "sync-full-resync should return structured result or safe configuration error, got {full_resync_status:?}",
    );
    let body: Value = serde_json::from_slice(
        &to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .expect("read sync-full-resync"),
    )
    .expect("sync-full-resync json");
    if full_resync_status == StatusCode::BAD_REQUEST {
        assert_eq!(body["error"], "invalid_communication_query");
        return;
    }
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
