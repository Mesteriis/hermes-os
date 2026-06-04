use std::env;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use hermes_hub_backend::storage::Database;
use hermes_hub_backend::{build_router, build_router_with_database, config::AppConfig};

#[tokio::test]
async fn healthz_returns_ok_status_and_service_name() {
    let app = build_router(AppConfig::default());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(
        value,
        json!({
            "status": "ok",
            "service": "hermes-hub-backend"
        })
    );
}

#[tokio::test]
async fn readyz_returns_service_unavailable_when_database_is_not_configured() {
    let app = build_router(AppConfig::default());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = to_bytes(response.into_body(), 2048)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(
        value,
        json!({
            "status": "degraded",
            "service": "hermes-hub-backend",
            "checks": {
                "database": {
                    "status": "not_configured",
                    "message": "DATABASE_URL is not configured"
                },
                "migrations": {
                    "status": "not_configured",
                    "message": "DATABASE_URL is not configured"
                }
            }
        })
    );
}

#[tokio::test]
async fn readyz_reports_database_and_migrations_ok_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live readiness test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(AppConfig::default(), database);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(value["status"], "ok");
    assert_eq!(value["checks"]["database"]["status"], "ok");
    assert_eq!(
        value["checks"]["database"]["message"],
        "database is reachable"
    );
    assert_eq!(value["checks"]["migrations"]["status"], "ok");
    assert_eq!(
        value["checks"]["migrations"]["message"],
        "required database migrations are applied"
    );
}
