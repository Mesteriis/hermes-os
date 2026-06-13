use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "persons-api-test-token";

fn config_with_api_token() -> AppConfig {
    app_config_with_pairs(Vec::new())
}

fn app_config_with_pairs(mut extra_pairs: Vec<(&'static str, String)>) -> AppConfig {
    let suffix = unique_suffix();
    let vault_home = format!("/tmp/hermes-persons-api-vault-{suffix}");
    let dev_key_path = format!("{vault_home}/dev.key");
    let mut pairs = vec![
        ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN.to_owned()),
        ("HERMES_DEV_MODE", "true".to_owned()),
        ("HERMES_VAULT_HOME", vault_home),
        ("HERMES_DEV_KEY_PATH", dev_key_path),
    ];
    pairs.append(&mut extra_pairs);
    AppConfig::from_pairs(pairs).expect("valid local API config")
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
        app_config_with_pairs(vec![("DATABASE_URL", database_url.to_owned())]),
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

#[tokio::test]
async fn personas_routes_return_persona_native_schema_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live personas route test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    sqlx::query("UPDATE persons SET is_self = false WHERE is_self = true")
        .execute(&pool)
        .await
        .expect("clear existing owner persona");
    let store = PersonProjectionStore::new(pool);
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_person(&format!("persona-native-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    store
        .set_owner_persona(&owner.person_id)
        .await
        .expect("set owner persona");

    let app = build_router_with_database(
        app_config_with_pairs(vec![("DATABASE_URL", database_url.to_owned())]),
        database,
    );

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/personas?limit=20",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("personas list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items array");
    assert!(
        items
            .iter()
            .any(|item| item["persona_id"] == owner.person_id && item["is_self"] == true),
        "personas list should include owner Persona: {body}"
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona detail response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.person_id);
    assert_eq!(body["persona_type"], "human");
    assert_eq!(body["is_self"], true);
    assert_eq!(body["identity"]["display_name"], owner.display_name);
    assert_eq!(body["identity"]["email_address"], owner.email_address);
    assert_eq!(body["communication"]["primary_email"], owner.email_address);
    assert_eq!(body["compatibility"]["legacy_person_id"], owner.person_id);
    assert_eq!(body["compatibility"]["legacy_route"], "/api/v1/persons");
}

#[tokio::test]
async fn personas_put_updates_compatibility_projection_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live personas write route test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    sqlx::query("UPDATE persons SET is_self = false WHERE is_self = true")
        .execute(&pool)
        .await
        .expect("clear existing owner persona");
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_person(&format!("persona-native-write-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    let previous_owner = store
        .upsert_email_person(&format!("persona-native-write-prev-{suffix}@example.com"))
        .await
        .expect("upsert previous owner persona");
    store
        .set_owner_persona(&previous_owner.person_id)
        .await
        .expect("set previous owner persona");

    let app = build_router_with_database(
        app_config_with_pairs(vec![("DATABASE_URL", database_url.to_owned())]),
        database,
    );

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            json!({
                "identity": {
                    "display_name": "Owner Persona"
                },
                "is_self": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona update response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.person_id);
    assert_eq!(body["identity"]["display_name"], "Owner Persona");
    assert_eq!(body["is_self"], true);

    let row = sqlx::query(
        r#"
        SELECT display_name, is_self
        FROM persons
        WHERE person_id = $1
        "#,
    )
    .bind(&owner.person_id)
    .fetch_one(&pool)
    .await
    .expect("updated persona row");
    assert_eq!(
        row.try_get::<String, _>("display_name").unwrap(),
        "Owner Persona"
    );
    assert!(row.try_get::<bool, _>("is_self").unwrap());

    let previous_is_self: bool =
        sqlx::query_scalar("SELECT is_self FROM persons WHERE person_id = $1")
            .bind(&previous_owner.person_id)
            .fetch_one(&pool)
            .await
            .expect("previous owner row");
    assert!(!previous_is_self);

    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            json!({ "is_self": false }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona unset owner response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn person_dossier_get_persists_snapshot_and_review_state_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live dossier snapshot API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = store
        .upsert_email_person(&format!("dossier-snapshot-{suffix}@example.com"))
        .await
        .expect("upsert dossier persona");

    let app = build_router_with_database(
        app_config_with_pairs(vec![("DATABASE_URL", database_url.to_owned())]),
        database,
    );

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/persons/{}/dossier",
                urlencoding_percent_encode(&person.person_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dossier response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let snapshot_id = body["dossier_snapshot_id"]
        .as_str()
        .expect("dossier snapshot id")
        .to_owned();
    assert_eq!(body["review_state"], "suggested");
    assert_eq!(body["person"]["person_id"], person.person_id);

    let row = sqlx::query(
        r#"
        SELECT persona_id, review_state, dossier
        FROM persona_dossier_snapshots
        WHERE dossier_snapshot_id = $1
        "#,
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("stored dossier snapshot");
    assert_eq!(
        row.try_get::<String, _>("persona_id").expect("persona id"),
        person.person_id
    );
    assert_eq!(
        row.try_get::<String, _>("review_state")
            .expect("review state"),
        "suggested"
    );
    let stored_dossier = row.try_get::<Value, _>("dossier").expect("dossier json");
    assert_eq!(stored_dossier["person"]["person_id"], person.person_id);

    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/persons/{}/dossier/review",
                urlencoding_percent_encode(&person.person_id)
            ),
            json!({ "review_state": "user_confirmed" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dossier review response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["dossier_snapshot_id"], snapshot_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert!(body["reviewed_at"].is_string());
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

#[tokio::test]
async fn person_owner_get_and_put_uses_owner_persona_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live owner persona API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    sqlx::query("UPDATE persons SET is_self = false WHERE is_self = true")
        .execute(&pool)
        .await
        .expect("clear existing owner persona");
    let store = PersonProjectionStore::new(pool);
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_person(&format!("owner-api-{suffix}@example.com"))
        .await
        .expect("upsert owner candidate");
    let other = store
        .upsert_email_person(&format!("not-owner-api-{suffix}@example.com"))
        .await
        .expect("upsert non-owner candidate");

    let app = build_router_with_database(
        app_config_with_pairs(vec![("DATABASE_URL", database_url.to_owned())]),
        database,
    );

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/persons/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("initial owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert!(body["owner_persona"].is_null());

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            "/api/v1/persons/owner",
            json!({ "person_id": owner.person_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("set owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["person_id"], owner.person_id);
    assert_eq!(body["owner_persona"]["is_self"], true);
    assert_eq!(body["owner_persona"]["persona_type"], "human");

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["person_id"], owner.person_id);
    assert_ne!(body["owner_persona"]["person_id"], other.person_id);
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
            let pid = format!("person:nonexistent-{suffix}");
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
    let pid = format!("person:nonexistent-{suffix}");
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
    let pid = format!("person:nonexistent-{suffix}");
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
    let pid = format!("person:nonexistent-{suffix}");
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
    let pid = format!("person:nonexistent-{suffix}");
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

#[tokio::test]
async fn identity_traces_create_list_and_attach_unattached_trace() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live identity traces API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;

    let create = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/identity-traces",
            json!({
                "identity_type": "message_participant",
                "identity_value": format!("message:v1:{suffix}:api-unattached"),
                "source": "communication_projection"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create trace response");
    assert_eq!(create.status(), StatusCode::OK);
    let create_body = json_body(create).await;
    assert_eq!(create_body["person_id"], Value::Null);
    assert_eq!(create_body["identity_type"], "message_participant");
    assert_eq!(create_body["source"], "communication_projection");
    let identity_id = create_body["id"].as_str().expect("identity id").to_owned();

    let list = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/identity-traces?status=unattached",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("list traces response");
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = json_body(list).await;
    let items = list_body["items"].as_array().expect("items");
    assert!(items.iter().any(|item| item["id"] == identity_id
        && item["person_id"] == Value::Null
        && item["identity_type"] == "message_participant"));

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool);
    let person = person_store
        .upsert_email_person(&format!("identity-trace-api-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let attach = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/identity-traces/{}/assignment",
                urlencoding_percent_encode(&identity_id)
            ),
            json!({ "person_id": person.person_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("attach trace response");
    assert_eq!(attach.status(), StatusCode::OK);
    let attach_body = json_body(attach).await;
    assert_eq!(attach_body["id"], identity_id);
    assert_eq!(attach_body["person_id"], person.person_id);
    assert_eq!(attach_body["status"], "active");
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
    let pid = format!("person:nonexistent-{suffix}");
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
