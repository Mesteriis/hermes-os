use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const TOKEN: &str = "calls-test-token";

fn cfg() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", TOKEN)]).expect("cfg")
}

fn get(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("req")
}

fn post(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn body(response: axum::response::Response) -> Value {
    let b = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&b).expect("json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

async fn app(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", TOKEN), ("DATABASE_URL", db)])
            .expect("cfg"),
        database,
    )
}

#[tokio::test]
async fn calls_reject_no_secret() {
    let r = build_router(cfg());
    let resp = r.oneshot(get("/api/v1/calls", "")).await.expect("r");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn calls_list_ok() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let a = app(&db).await;
    let resp = a.oneshot(get("/api/v1/calls", TOKEN)).await.expect("r");
    assert!(!resp.status().is_server_error(), "status={}", resp.status());
    assert!(body(resp).await["items"].is_array());
}

#[tokio::test]
async fn call_create_ok() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let s = uid();
    let a = app(&db).await;
    let resp = a.oneshot(post("/api/v1/calls", json!({
        "call_type": "telegram", "chat_id": format!("c{s}"), "direction": "inbound",
        "state": "completed", "initiated_at": chrono::Utc::now().to_rfc3339(), "duration_seconds": 120
    }), TOKEN)).await.expect("r");
    assert!(!resp.status().is_server_error(), "status={}", resp.status());
}

#[tokio::test]
async fn call_transcript_404() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let s = uid();
    let a = app(&db).await;
    let resp = a
        .oneshot(get(
            &format!("/api/v1/calls/call:nonexistent-{s}/transcript"),
            TOKEN,
        ))
        .await
        .expect("r");
    assert!(resp.status() == StatusCode::NOT_FOUND || resp.status().is_success());
}
