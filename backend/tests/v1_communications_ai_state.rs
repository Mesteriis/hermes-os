use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const T: &str = "v1comms-ai-state-test-token";

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", T)
        .body(Body::empty())
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

#[tokio::test]
async fn v1_message_ai_state_transitions_are_durable_and_emit_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-ai-state-api-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-ai-state-api-{suffix}"),
        "AI state transition",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages/{message_id}/ai-state"
        )))
        .await
        .expect("initial ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["ai_state"], "NEW");

    let response = r
        .clone()
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/ai-state"),
            json!({"ai_state": "PROCESSING"}),
        ))
        .await
        .expect("transition ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["ai_state"], "PROCESSING");
    assert!(body["updated_at"].is_string());

    let persisted = sqlx::query(
        r#"
        SELECT ai_state, review_reason, last_error
        FROM mail_ai_states
        WHERE message_id = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("persisted ai state");
    assert_eq!(
        persisted.try_get::<String, _>("ai_state").unwrap(),
        "PROCESSING"
    );
    assert!(
        persisted
            .try_get::<Option<String>, _>("review_reason")
            .unwrap()
            .is_none()
    );
    assert!(
        persisted
            .try_get::<Option<String>, _>("last_error")
            .unwrap()
            .is_none()
    );

    let event = sqlx::query(
        r#"
        SELECT subject, payload
        FROM event_log
        WHERE event_type = 'mail.ai_state.changed'
          AND subject->>'kind' = 'mail_ai_state'
          AND subject->>'id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("ai state event");
    let subject = event.try_get::<Value, _>("subject").unwrap();
    let payload = event.try_get::<Value, _>("payload").unwrap();
    assert_eq!(subject["message_id"], message_id);
    assert_eq!(payload["ai_state"], "PROCESSING");
    assert_eq!(payload["previous_ai_state"], "NEW");
    assert!(payload.get("body_text").is_none());

    let response = r
        .oneshot(get(&format!(
            "/api/v1/communications/messages/{message_id}/ai-state"
        )))
        .await
        .expect("current ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["ai_state"], "PROCESSING");
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
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
                "body_text": "Private body that must not be emitted in AI state events"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
