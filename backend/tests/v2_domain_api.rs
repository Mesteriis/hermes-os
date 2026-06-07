use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::domains::tasks::api::{NewTask, TaskStore};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const LOCAL_API_TOKEN: &str = "v2-domain-api-test-token";

#[tokio::test]
async fn domain_routes_build_and_require_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/tasks"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        json_body(response).await,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-hermes-secret header"
        })
    );

    let secret_only_response = build_router(config_with_api_token())
        .oneshot(get_request_with_token("/api/v1/tasks", LOCAL_API_TOKEN))
        .await
        .expect("secret-only response");

    assert_eq!(
        secret_only_response.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        json_body(secret_only_response).await,
        json!({
            "error": "database_not_configured",
            "message": "DATABASE_URL is not configured"
        })
    );
}

#[tokio::test]
async fn tasks_endpoint_returns_first_class_task_payload_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live tasks API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let task = TaskStore::new(pool)
        .create(&NewTask {
            title: format!("V1 first-class task {suffix}"),
            description: Some("contract test task".to_owned()),
            source_kind: Some("manual".to_owned()),
            source_id: Some(format!("manual-v1-task-{suffix}")),
            source_type: Some("manual".to_owned()),
            hermes_status: Some("ready".to_owned()),
            priority_score: Some(0.7),
            tags: Some(json!(["api-test"])),
            ..Default::default()
        })
        .await
        .expect("seed task");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/tasks?limit=100",
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let item = body["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["task_id"] == task.task_id)
        .expect("seeded task item");

    assert_eq!(item["title"], json!(task.title));
    assert_eq!(item["source_type"], json!("manual"));
    assert_eq!(item["hermes_status"], json!("ready"));
    assert_eq!(item["confidentiality"], json!("private_local"));
    assert_eq!(item["task_metadata"], json!({}));
}

#[tokio::test]
async fn person_health_endpoint_returns_single_person_health_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person health API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&format!("health-{suffix}@example.com"))
        .await
        .expect("seed person");
    seed_person_health(&pool, &person.person_id).await;

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(get_request_with_token_and_actor(
            &format!("/api/v1/persons/{}/health", person.person_id),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["person_id"], json!(person.person_id));
    assert_eq!(body["health_status"], json!("at_risk"));
    assert_eq!(body["communication_gap_days"], json!(42));
    assert!(body.get("items").is_none());
}

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

fn get_request_with_token_and_actor(uri: &str, token: &str, _actor_id: &str) -> Request<Body> {
    Request::builder()
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

async fn seed_person_health(pool: &PgPool, person_id: &str) {
    sqlx::query(
        "UPDATE persons SET health_status = 'at_risk', communication_gap_days = 42, watchlist = true WHERE person_id = $1",
    )
    .bind(person_id)
    .execute(pool)
    .await
    .expect("update person health");
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}
