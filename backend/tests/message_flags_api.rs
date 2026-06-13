use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use chrono::Utc;
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{MessageProjectionStore, NewProjectedMessage};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const TOKEN: &str = "message-flags-api-test-token";

async fn app(ctx: &TestContext) -> axum::Router {
    let database = Database::connect(Some(&ctx.connection_string()))
        .await
        .expect("database");
    build_router_with_database(
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", TOKEN),
            ("DATABASE_URL", ctx.connection_string().as_str()),
        ])
        .expect("config"),
        database,
    )
}

fn request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-hermes-secret", TOKEN)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("json")
}

#[tokio::test]
async fn message_important_endpoint_toggles_metadata_flag() {
    let ctx = TestContext::new().await;
    let communication_store = CommunicationIngestionStore::new(ctx.pool().clone());
    let message_store = MessageProjectionStore::new(ctx.pool().clone());
    let suffix = unique_suffix();
    let account_id = format!("acct-important-{suffix}");
    let raw_record_id = format!("raw-important-{suffix}");
    let provider_record_id = format!("provider-important-{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Important IMAP",
            format!("important-{suffix}@example.com"),
        ))
        .await
        .expect("account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "subject": "Important subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Important body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"message_flags_api_test"})),
        )
        .await
        .expect("raw record");

    let projected = message_store
        .upsert_message(&NewProjectedMessage {
            message_id: format!("message-important-{suffix}"),
            raw_record_id: raw.raw_record_id,
            account_id: account_id.clone(),
            provider_record_id,
            subject: "Important subject".to_owned(),
            sender: "alice@example.com".to_owned(),
            recipients: vec!["bob@example.com".to_owned()],
            body_text: "Important body".to_owned(),
            occurred_at: raw.occurred_at,
            channel_kind: "email".to_owned(),
            conversation_id: None,
            sender_display_name: Some("alice@example.com".to_owned()),
            delivery_state: "received".to_owned(),
            message_metadata: json!({}),
        })
        .await
        .expect("message");
    let message_id = projected.message_id;

    let app = app(&ctx).await;
    let uri = format!("/api/v1/communications/messages/{message_id}/important");

    let response = app
        .clone()
        .oneshot(request(Method::POST, &uri))
        .await
        .expect("important response");
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["important"], true);
    let stored = message_store
        .message(&message_id)
        .await
        .expect("stored message")
        .expect("message exists");
    assert_eq!(stored.message_metadata["important"], true);

    let response = app
        .oneshot(request(Method::POST, &uri))
        .await
        .expect("second important response");
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["important"], false);
    let stored = message_store
        .message(&message_id)
        .await
        .expect("stored message")
        .expect("message exists");
    assert_eq!(stored.message_metadata["important"], false);
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
        .to_string()
}
