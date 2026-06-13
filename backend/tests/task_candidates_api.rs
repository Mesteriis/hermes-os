use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::domains::tasks::candidates::TaskCandidateStore;
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "task-candidates-api-test-token";

#[tokio::test]
async fn task_candidates_reject_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/task-candidates"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-hermes-secret header"
        })
    );
}

#[tokio::test]
async fn task_candidates_returns_safe_candidate_payload() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live task candidate API test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let keyword = format!("TaskCandidatesApi{suffix}");

    let store = TaskCandidateStore::new(pool.clone());
    let message_id = seed_message(
        &pool,
        suffix,
        &format!("api-message-{suffix}@example.com"),
        &[format!("api-recipient-{suffix}@example.com")],
        &format!("provider-task-candidate-api-msg-{suffix}"),
        &format!("Task API {keyword}"),
        "Please follow up with the client",
    )
    .await;
    let document_id = seed_document(
        &pool,
        &format!("document_task_candidate_api_{suffix}"),
        &format!("{keyword} plan"),
        "Please review this task",
    )
    .await;
    let _ = store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh candidates");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/task-candidates?limit={}&", 100),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty());

    let message_payload = items
        .iter()
        .find(|item| item["source_id"] == json!(message_id))
        .expect("message payload");
    let document_payload = items
        .iter()
        .find(|item| item["source_id"] == json!(document_id))
        .expect("document payload");
    assert_eq!(message_payload["source_kind"], "message");
    assert_eq!(document_payload["source_kind"], "document");
    assert!(message_payload["evidence_excerpt"].is_string());
    assert!(document_payload["evidence_excerpt"].is_string());
    assert!(message_payload.get("candidate_kind").is_none());
    assert!(message_payload.get("candidate_metadata").is_none());
    assert!(document_payload.get("candidate_kind").is_none());
    assert!(document_payload.get("candidate_metadata").is_none());
}

#[tokio::test]
async fn put_task_candidate_review_confirms_task() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live task candidate review API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let store = TaskCandidateStore::new(pool.clone());

    let message_id = seed_message(
        &pool,
        suffix,
        &format!("review-api-{suffix}@example.com"),
        &[format!("api-owner-{suffix}@example.com")],
        &format!("provider-task-candidate-review-api-{suffix}"),
        &format!("Task review API {suffix}"),
        "Action: process this ticket",
    )
    .await;
    let _ = store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh candidates");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'message'",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("candidate id");

    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let command_id = format!("task-candidate-api-confirm-{suffix}");
    let response = app
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/task-candidates/{task_candidate_id}/review"),
            json!({
                "command_id": command_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "task_candidate_id": task_candidate_id,
            "review_state": "user_confirmed",
            "event_id": format!("task_candidate_review:{command_id}"),
        })
    );
}

#[tokio::test]
async fn put_task_candidate_review_rejects_missing_candidate() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live task candidate missing candidate API test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url.as_str()),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .oneshot(json_put_request_with_actor(
            "/api/v1/task-candidates/task_candidate:v1:missing%3Amissing%3Acandidate/review",
            json!({
                "command_id": "task-candidate-missing-review",
                "review_state": "user_rejected",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "task_candidate_not_found",
            "message": "task candidate was not found"
        })
    );
}

#[derive(Clone)]
struct TaskCandidatesApiContext {
    communication_store: CommunicationIngestionStore,
    message_store: MessageProjectionStore,
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

fn json_put_request_with_actor(uri: &str, value: Value, token: &str) -> Request<Body> {
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

async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let context = TaskCandidatesApiContext {
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
    };
    let account_id = format!("acct_task_candidate_api_{suffix}");
    context
        .communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Task Candidates API Gmail",
            format!("api-task-candidate-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_task_candidate_api_{suffix}_{provider_record_id}");
    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:task-candidate-api:{suffix}:{provider_record_id}"),
                format!("batch-task-candidate-api_{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(chrono::Utc::now())
            .provenance(json!({"source":"task_candidates_api_test"})),
        )
        .await
        .expect("record raw message");

    project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

async fn seed_document(pool: &PgPool, document_id: &str, title: &str, body: &str) -> String {
    let import = NewDocumentImport::markdown(document_id, title, body);
    DocumentImportStore::new(pool.clone())
        .import_document(&import)
        .await
        .expect("document import");
    document_id.to_owned()
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
