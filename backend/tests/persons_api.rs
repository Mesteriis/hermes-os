use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "persons-api-test-token";

fn config_with_api_token() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN)])
        .expect("valid local API secret")
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn put_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

async fn build_persons_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url),
        ])
        .expect("config"),
        database,
    )
}

// ── Auth ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn persons_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/persons"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({"error": "invalid_api_secret", "message": "missing or invalid x-hermes-secret header"})
    );
}

// ── Persons List ───────────────────────────────────────────────────────────

#[tokio::test]
async fn persons_list_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live persons list test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token("/api/v1/persons", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Person Detail ──────────────────────────────────────────────────────────

#[tokio::test]
async fn person_detail_not_found_returns_404() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person detail test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/persons/person:nonexistent-{suffix}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── Person Search ──────────────────────────────────────────────────────────

#[tokio::test]
async fn person_search_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person search test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/search?q=alex",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Read-only person endpoints (resilient to missing person) ───────────────

macro_rules! person_endpoint_test {
    ($name:ident, $path_suffix:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!(
                    "skipping live {} test: HERMES_TEST_DATABASE_URL is not set",
                    stringify!($name)
                );
                return;
            };
            let suffix = unique_suffix();
            let app = build_persons_app(&database_url).await;
            let path = format!(
                "/api/v1/persons/person:nonexistent-{}{}",
                suffix, $path_suffix
            );
            let response = app
                .oneshot(get_request_with_token(&path, LOCAL_API_TOKEN))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "status={}",
                response.status()
            );
        }
    };
}

person_endpoint_test!(person_identities_list, "/identities");
person_endpoint_test!(person_roles_list, "/roles");
person_endpoint_test!(person_personas_list, "/personas");
person_endpoint_test!(person_facts_list, "/facts");
person_endpoint_test!(person_memory_cards_list, "/memory-cards");
person_endpoint_test!(person_preferences_list, "/preferences");
person_endpoint_test!(person_timeline_list, "/timeline");
person_endpoint_test!(person_snapshots_list, "/snapshots");
person_endpoint_test!(person_history_diff, "/history-diff");
person_endpoint_test!(person_enrichment_list, "/enrichment");
person_endpoint_test!(person_expertise_list, "/expertise");
person_endpoint_test!(person_promises_list, "/promises");
person_endpoint_test!(person_risks_list, "/risks");
person_endpoint_test!(person_health_get, "/health");
person_endpoint_test!(person_dossier_get, "/dossier");
person_endpoint_test!(person_meeting_prep_get, "/meeting-prep");
person_endpoint_test!(person_analytics_get, "/analytics");
person_endpoint_test!(person_export_get, "/export");
person_endpoint_test!(person_identity_detail, "/identity");

// ── Persons Health ─────────────────────────────────────────────────────────

#[tokio::test]
async fn persons_health_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live persons health test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/health",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Persons Watchlist ──────────────────────────────────────────────────────

#[tokio::test]
async fn persons_watchlist_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live persons watchlist test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/watchlist",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Identity Candidates ────────────────────────────────────────────────────

#[tokio::test]
async fn identity_candidates_list_returns_ok() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live identity candidates test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/identity-candidates",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Person write endpoints (exercise handlers, may fail gracefully) ────────

macro_rules! person_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip");
                return;
            };
            let suffix = unique_suffix();
            let app = build_persons_app(&database_url).await;
            let pid = format!("person:nonexistent-{}", suffix);
            let r = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/persons/{}/{}",
                        urlencoding_percent_encode(&pid),
                        $path_suffix
                    ),
                    $body,
                    LOCAL_API_TOKEN,
                ))
                .await
                .expect("r");
            assert!(
                !r.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                r.status()
            );
        }
    };
}

person_post_test!(
    person_post_fingerprint,
    "fingerprint",
    json!({"fingerprint_data": "test-fingerprint-data"})
);
person_post_test!(person_post_favorite, "favorite", json!({}));
person_post_test!(
    person_post_investigate,
    "investigate",
    json!({"query": "background check"})
);

#[tokio::test]
async fn person_put_notes() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{}", suffix);
    let r = app
        .oneshot(put_request_with_token(
            &format!("/api/v1/persons/{}/notes", urlencoding_percent_encode(&pid)),
            json!({"notes": "Test notes content"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "notes status={}", r.status());
}

// Person Expertise Search
#[tokio::test]
async fn person_expertise_search() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let app = build_persons_app(&database_url).await;
    let r = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/search/expertise?q=rust",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "expertise search={}",
        r.status()
    );
}

// Person Identity candidates review
#[tokio::test]
async fn identity_candidate_review() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let r = app
        .oneshot(put_request_with_token(
            &format!("/api/v1/identity-candidates/ic:fake-{suffix}/review"),
            json!({"review_state": "declined", "reason": "Not a match"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "identity review={}",
        r.status()
    );
}

// Person Roles CRUD (POST + DELETE)
#[tokio::test]
async fn person_roles_post_and_delete() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{}", suffix);
    let r = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{}/roles", urlencoding_percent_encode(&pid)),
            json!({"role": "colleague", "organization": "TestCo"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "role post={}", r.status());

    let r = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/persons/{}/roles/colleague",
                urlencoding_percent_encode(&pid)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "role delete={}", r.status());
}

// Person Persona POST + DELETE
#[tokio::test]
async fn person_persona_post_and_delete() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{}", suffix);
    let r = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{}/personas",
                urlencoding_percent_encode(&pid)
            ),
            json!({"name": "Work Persona", "description": "Professional context"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "persona post={}", r.status());

    let r = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/persons/{}/personas/pers:fake",
                urlencoding_percent_encode(&pid)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "persona delete={}",
        r.status()
    );
}

// Person Identity POST + DELETE
#[tokio::test]
async fn person_identity_post_and_delete() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{}", suffix);
    let r = app.clone().oneshot(post_request_with_token(
        &format!("/api/v1/persons/{}/identities", urlencoding_percent_encode(&pid)),
        json!({"identity_type": "email", "identity_value": format!("test-{suffix}@example.com"), "source": "manual"}),
        LOCAL_API_TOKEN,
    )).await.expect("r");
    assert!(
        !r.status().is_server_error(),
        "identity post={}",
        r.status()
    );

    let r = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/persons/{}/identities/id:fake",
                urlencoding_percent_encode(&pid)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "identity delete={}",
        r.status()
    );
}

// Person Memory POSTs
person_post_test!(
    person_post_fact,
    "facts",
    json!({"fact_type": "preference", "value": "Likes coffee", "confidence": 0.9})
);
person_post_test!(
    person_post_memory_card,
    "memory-cards",
    json!({"title": "Memory card", "content": "Test memory content"})
);
person_post_test!(
    person_post_preference,
    "preferences",
    json!({"key": "communication_style", "value": "direct"})
);
person_post_test!(
    person_post_timeline,
    "timeline",
    json!({"event_type": "meeting", "description": "Test meeting", "occurred_at": "2027-01-01T00:00:00Z"})
);

// Person Watchlist Toggle
#[tokio::test]
async fn person_watchlist_toggle() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{}", suffix);
    let r = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{}/watchlist",
                urlencoding_percent_encode(&pid)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("r");
    assert!(
        !r.status().is_server_error(),
        "watchlist toggle={}",
        r.status()
    );
}
