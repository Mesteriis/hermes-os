use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use hermes_hub_backend::build_router_with_database;
use hermes_hub_backend::config::AppConfig;
use hermes_hub_backend::document_processing::DocumentProcessingStore;
use hermes_hub_backend::documents::{DocumentImportStore, NewDocumentImport};
use hermes_hub_backend::storage::Database;
use serde_json::Value;
use sqlx::query_scalar;
use tower::ServiceExt;

const LOCAL_API_TOKEN: &str = "document-processing-api-test-token";
const LOCAL_API_ACTOR_ID: &str = "document-processing-api-test-client";
const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";

#[tokio::test]
async fn get_document_processing_jobs_rejects_missing_local_api_token() {
    let app = hermes_hub_backend::build_router(
        AppConfig::from_pairs([("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN)]).expect("config"),
    );

    let response = app
        .oneshot(get_request("/api/v2/document-processing/jobs"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "invalid_api_token",
            "message": "missing or invalid bearer token"
        })
    );
}

#[tokio::test]
async fn get_document_processing_for_missing_document_returns_404() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing API missing-document test: HERMES_TEST_DATABASE_URL is not set"
        );
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
    let missing_document_id = format!("doc_processing_api_missing_{:x}", unique_suffix());

    let response = app
        .oneshot(get_request_with_actor(
            &format!("/api/v2/documents/{missing_document_id}/processing"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "document_processing_store_error",
            "message": "document processing job was not found"
        })
    );
}

#[tokio::test]
async fn document_processing_api_returns_expected_payloads() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live document processing API payload test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let document_id = format!("doc_processing_api_{:x}", unique_suffix());

    let document_store = DocumentImportStore::new(pool.clone());
    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "pipeline api doc",
            "# Pipeline API\nMarkdown body for API test.",
        ))
        .await
        .expect("import markdown document");

    let processing_store = DocumentProcessingStore::new(pool.clone());
    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_TOKEN", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let jobs_response = app
        .clone()
        .oneshot(get_request_with_actor(
            "/api/v2/document-processing/jobs?limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(jobs_response.status(), StatusCode::OK);
    let jobs_body = json_body(jobs_response).await;
    let items = jobs_body["items"]
        .as_array()
        .expect("document processing jobs");
    assert!(!items.is_empty());
    let has_target = items
        .iter()
        .any(|item| item["document_id"] == Value::String(document_id.clone()));
    assert!(has_target, "jobs should include enqueued document");

    let detail_response = app
        .oneshot(get_request_with_actor(
            &format!("/api/v2/documents/{document_id}/processing"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = json_body(detail_response).await;
    assert_eq!(
        detail_body["document_id"],
        Value::String(document_id.clone())
    );
    assert!(detail_body["jobs"].is_array());
    assert!(detail_body["jobs"].as_array().unwrap().len() >= 2);

    let _ = query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM document_processing_jobs WHERE document_id = $1)",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("jobs inserted for processing document");
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_actor(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(LOCAL_API_ACTOR_ID_HEADER, LOCAL_API_ACTOR_ID)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
