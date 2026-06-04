use axum::body::{Body, to_bytes};
use axum::http::{HeaderValue, Request, StatusCode, header};
use serde_json::json;
use tower::ServiceExt;

use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::storage::Database;
use hermes_hub_backend::{build_router, build_router_with_database};

const LOCAL_API_TOKEN: &str = "test-token";
const LOCAL_API_ACTOR_ID: &str = "test-actor";
const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";

#[tokio::test]
async fn v1_status_returns_enabled_surfaces_against_postgres() {
    let Some(database_url) = std::env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live v1 API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/status")
                .header(
                    header::AUTHORIZATION,
                    HeaderValue::from_static("Bearer test-token"),
                )
                .header(
                    LOCAL_API_ACTOR_ID_HEADER,
                    HeaderValue::from_static(LOCAL_API_ACTOR_ID),
                )
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");
    assert_eq!(value["version"], json!("1.0"));
    assert_eq!(value["surfaces"]["messages"], json!(true));
    assert_eq!(value["surfaces"]["contacts"], json!(true));
    assert_eq!(value["surfaces"]["search"], json!(true));
    assert_eq!(value["surfaces"]["documents"], json!(true));
}

#[tokio::test]
async fn v1_status_rejects_missing_local_api_token_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/status"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_token",
            "message": "missing or invalid bearer token"
        })
    );
}

#[tokio::test]
async fn v1_status_rejects_invalid_local_api_token_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/status",
            "wrong-token",
            LOCAL_API_ACTOR_ID,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_token",
            "message": "missing or invalid bearer token"
        })
    );
}

#[tokio::test]
async fn v1_status_rejects_missing_local_api_actor_id_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_without_actor(
            "/api/v1/status",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_actor_id",
            "message": "missing or invalid x-hermes-actor-id header"
        })
    );
}

#[tokio::test]
async fn v1_status_rejects_invalid_local_api_actor_id_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/status",
            LOCAL_API_TOKEN,
            "invalid actor",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_actor_id",
            "message": "missing or invalid x-hermes-actor-id header"
        })
    );
}

#[tokio::test]
async fn v1_status_returns_service_unavailable_after_auth_when_database_is_not_configured() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/status",
            LOCAL_API_TOKEN,
            LOCAL_API_ACTOR_ID,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "database_not_configured",
            "message": "DATABASE_URL is not configured"
        })
    );
}

fn config_with_api_token() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN)])
        .expect("valid local API token")
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token_without_actor(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token_and_actor(uri: &str, token: &str, actor_id: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, actor_id)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}
