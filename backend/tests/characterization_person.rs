//! Characterization tests for Person / Persona API.
//!
//! Captures CURRENT behavior before alignment refactoring (Phase 2+).
//! Do NOT change existing behavior — only add tests.
//!
//! These tests rely on HERMES_TEST_DATABASE_URL pointing to a running
//! pgvector instance with migrations applied.

use std::env;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const TOKEN: &str = "char-person-test-token";

fn cfg(db: &str) -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", TOKEN), ("DATABASE_URL", db)]).expect("cfg")
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", TOKEN)
        .body(Body::empty())
        .expect("req")
}

fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", TOKEN)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

async fn build_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(cfg(database_url), database)
}

fn require_db() -> String {
    env::var("HERMES_TEST_DATABASE_URL")
        .expect("HERMES_TEST_DATABASE_URL must be set for integration tests")
}

// ── AC2: Person API characterization ────────────────────────────────────────

/// GAP-2 characterisation: GET /api/v1/persons returns 200 with items array.
#[tokio::test]
async fn char_persons_list_returns_ok() {
    let db = require_db();
    let app = build_app(&db).await;

    let response = app.oneshot(get("/api/v1/persons")).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert!(
        body.get("items").is_some(),
        "GET /api/v1/persons must return 'items' array"
    );

    // Characterize current pagination structure
    if let Some(items) = body["items"].as_array() {
        assert!(
            items.len() <= 50,
            "default persons limit should be <= 50, got {}",
            items.len()
        );
    }
}

/// GAP-2 characterisation: GET /api/v1/personas returns persona-native schema.
#[tokio::test]
async fn char_personas_list_returns_ok() {
    let db = require_db();
    let app = build_app(&db).await;

    let response = app
        .oneshot(get("/api/v1/personas"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert!(
        body.get("items").is_some(),
        "GET /api/v1/personas must return 'items' array"
    );
}

/// GAP-2 characterisation: both /api/v1/persons and /api/v1/personas coexist.
#[tokio::test]
async fn char_persons_and_personas_both_exist() {
    let db = require_db();
    let app = build_app(&db).await;

    let persons_resp = app
        .clone()
        .oneshot(get("/api/v1/persons"))
        .await
        .expect("persons response");
    assert_eq!(persons_resp.status(), StatusCode::OK);

    let personas_resp = app
        .clone()
        .oneshot(get("/api/v1/personas"))
        .await
        .expect("personas response");
    assert_eq!(personas_resp.status(), StatusCode::OK);
}

/// GAP-2 characterisation: GET /api/v1/persons/owner returns owner persona.
#[tokio::test]
async fn char_owner_persona_returns_ok() {
    let db = require_db();
    let app = build_app(&db).await;

    let response = app
        .oneshot(get("/api/v1/persons/owner"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert!(
        body.get("persona_id").is_some() || body.get("is_self").is_some(),
        "Owner response should contain persona fields: {:?}",
        body
    );
}

/// GAP-2 characterisation: GET /api/v1/persons/search requires a 'q' param.
#[tokio::test]
async fn char_person_search_requires_query() {
    let db = require_db();
    let app = build_app(&db).await;

    let response = app
        .oneshot(get("/api/v1/persons/search"))
        .await
        .expect("response");
    // Expect 400 for empty search
    assert!(
        response.status().is_client_error(),
        "search without query should return 4xx, got {}",
        response.status()
    );
}

/// GAP-2 characterisation: GET /api/v1/persons/{id} returns person by ID.
#[tokio::test]
async fn char_person_by_id_returns_ok_or_404() {
    let db = require_db();
    let app = build_app(&db).await;

    // Using a non-existent person ID — expect 404
    let response = app
        .oneshot(get("/api/v1/persons/person:nonexistent"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// GAP-2 characterisation: PUT /api/v1/personas/{persona_id} updates persona.
#[tokio::test]
async fn char_persona_update_accepts_valid_body() {
    let db = require_db();
    let app = build_app(&db).await;

    // Non-existent persona — expect 404 or appropriate error
    let response = app
        .oneshot(put(
            "/api/v1/personas/persona:nonexistent",
            json!({"name": "Updated Name"}),
        ))
        .await
        .expect("response");
    assert!(
        response.status().is_client_error(),
        "updating non-existent persona should return 4xx, got {}",
        response.status()
    );
}
