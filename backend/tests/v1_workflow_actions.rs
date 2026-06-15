use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::{build_router, build_router_with_database};
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;

const T: &str = "v1-workflow-action-test-token";

fn cfg() -> AppConfig {
    AppConfig::from_pairs([("HERMES_LOCAL_API_SECRET", T)]).expect("config")
}

fn post_with_actor(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", T)
        .header("x-hermes-actor-id", "hermes-frontend")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", T),
            ("DATABASE_URL", database_url),
        ])
        .expect("config"),
        database,
    )
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

#[tokio::test]
async fn workflow_action_endpoint_exists_without_database() {
    let app = build_router(cfg());
    let response = app
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": "workflow-action-no-db",
                "action": "reply",
                "source": { "kind": "communication_message", "id": "msg:no-db" }
            }),
        ))
        .await
        .expect("workflow action no-db response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn v1_put_workflow_state() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let response = r
        .oneshot(put(
            "/api/v1/communications/messages/msg:fake/workflow-state",
            json!({"state": "reviewed"}),
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "workflow state={}",
        response.status()
    );
}

#[tokio::test]
async fn workflow_action_create_task_is_idempotent_and_records_safe_event() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip workflow action task: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-action-{suffix}"),
        &format!("provider-workflow-action-{suffix}"),
        &format!("Workflow action task {suffix}"),
    )
    .await;
    let r = router(&db).await;
    let command_id = format!("workflow-action-task-{suffix}");
    let body = json!({
        "command_id": command_id,
        "action": "create_task",
        "source": { "kind": "communication_message", "id": message_id },
        "input": { "title": "Confirm integration access" }
    });

    let first = r
        .clone()
        .oneshot(post_with_actor("/api/v1/workflow-actions", body.clone()))
        .await
        .expect("first workflow action response");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    assert_eq!(
        first_body["event_id"],
        json!(format!("workflow_action:{command_id}"))
    );
    assert_eq!(first_body["target"]["kind"], "task");
    assert_eq!(first_body["provenance"]["source_id"], message_id);

    let second = r
        .oneshot(post_with_actor("/api/v1/workflow-actions", body))
        .await
        .expect("second workflow action response");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    assert_eq!(second_body["target"], first_body["target"]);

    let task_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM tasks WHERE source_id = $1 AND source_type = 'message'",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("task count");
    assert_eq!(task_count, 1);

    let event_payload: Value =
        sqlx::query_scalar("SELECT payload FROM event_log WHERE event_id = $1")
            .bind(format!("workflow_action:{command_id}"))
            .fetch_one(&pool)
            .await
            .expect("workflow event payload");
    assert!(
        !event_payload
            .to_string()
            .contains("Body for local trash API")
    );
}

#[tokio::test]
async fn workflow_action_create_note_creates_markdown_document() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip workflow action note: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let r = router(&db).await;
    let command_id = format!("workflow-action-note-{suffix}");

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": command_id,
                "action": "create_note",
                "input": {
                    "title": "Follow-up note",
                    "body": "Remember to verify keys with the integration owner."
                }
            }),
        ))
        .await
        .expect("workflow note response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["target"]["kind"], "document");
    let document_id = body["target"]["id"].as_str().expect("document id");
    let document_kind: String =
        sqlx::query_scalar("SELECT document_kind FROM documents WHERE document_id = $1")
            .bind(document_id)
            .fetch_one(&pool)
            .await
            .expect("document kind");
    assert_eq!(document_kind, "markdown");
}

#[tokio::test]
async fn workflow_action_create_event_requires_start_and_end() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip workflow action event validation: no DB");
        return;
    };
    let r = router(&db).await;
    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": format!("workflow-action-event-missing-{}", uid()),
                "action": "create_event",
                "input": { "title": "Missing time event" }
            }),
        ))
        .await
        .expect("workflow event validation response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn workflow_action_archive_transitions_message_locally() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip workflow action archive: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-archive-{suffix}"),
        &format!("provider-workflow-archive-{suffix}"),
        &format!("Workflow archive {suffix}"),
    )
    .await;
    let r = router(&db).await;

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": format!("workflow-action-archive-{suffix}"),
                "action": "archive",
                "source": { "kind": "communication_message", "id": message_id }
            }),
        ))
        .await
        .expect("workflow archive response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["status"], "archived");
    let workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("workflow state");
    assert_eq!(workflow_state, "archived");
}

async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Seed Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            account_id,
            "email_message",
            provider_record_id,
            format!("sha256:{provider_record_id}"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": subject,
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Body for local trash API"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
