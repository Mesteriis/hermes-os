use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::engines::consistency::{
    ContradictionObservation, ContradictionObservationStore, ContradictionReviewState,
    ContradictionSeverity, ContradictionSourceKind, NewContradictionObservation,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "contradictions-api-test-token";

#[tokio::test]
async fn contradictions_list_returns_open_reviewable_observations() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contradictions API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_contradiction_observation(&pool, suffix).await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/contradictions?limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let item = items
        .iter()
        .find(|item| item["observation_id"] == json!(stored.observation_id))
        .expect("seeded contradiction observation");

    assert_eq!(item["conflict_type"], "direct_contradiction");
    assert_eq!(item["old_claim"], stored.old_claim);
    assert_eq!(item["new_claim"], stored.new_claim);
    assert_eq!(item["review_state"], "suggested");
}

#[tokio::test]
async fn put_contradiction_review_updates_review_state_without_overwriting_memory() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contradiction review API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_contradiction_observation(&pool, suffix).await;
    let observation_id = path_segment(&stored.observation_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/contradictions/{observation_id}/review"),
            json!({
                "review_state": "user_confirmed",
                "resolution": "confirmed from source review"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["observation_id"], stored.observation_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert_eq!(body["reviewed_by"], "hermes-frontend");
    assert_eq!(body["resolution"], "confirmed from source review");

    let row: (String, Option<String>, i64) = sqlx::query_as(
        r#"
        SELECT
            review_state,
            resolution,
            (
                SELECT count(*)
                FROM person_facts
                WHERE value = $2
            ) AS memory_overwrite_count
        FROM contradiction_observations
        WHERE observation_id = $1
        "#,
    )
    .bind(&stored.observation_id)
    .bind(&stored.new_claim)
    .fetch_one(&pool)
    .await
    .expect("stored contradiction review");

    assert_eq!(row.0, "user_confirmed");
    assert_eq!(row.1.as_deref(), Some("confirmed from source review"));
    assert_eq!(row.2, 0);
}

async fn app_and_pool(database_url: &str) -> (axum::Router, PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url),
        ])
        .expect("config"),
        database,
    );

    (app, pool)
}

async fn seed_contradiction_observation(pool: &PgPool, suffix: u128) -> ContradictionObservation {
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: format!("memory:contradiction-api:{suffix}"),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: format!("message:contradiction-api:{suffix}"),
        affected_entities: json!([
            {"entity_kind": "persona", "entity_id": format!("person:v1:email:polygraph-{suffix}@example.com")}
        ]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "status=available".to_owned(),
        new_claim: format!("status=unavailable-{suffix}"),
        confidence: 0.86,
        severity: ContradictionSeverity::Medium,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({"source": "contradictions_api_test"}),
    };

    ContradictionObservationStore::new(pool.clone())
        .upsert(&observation)
        .await
        .expect("seed contradiction observation")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", token)
        .body(Body::from(value.to_string()))
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

fn path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
