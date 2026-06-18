use std::env;
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

const T: &str = "v1comms-action-test-token";

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", T)
        .body(Body::empty())
        .expect("request")
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-hermes-secret", T)
        .body(Body::empty())
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

macro_rules! v1_msg_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
                eprintln!("skip");
                return;
            };
            let r = router(&db).await;
            let response = r
                .oneshot(post(
                    &format!("/api/v1/communications/messages/msg:fake/{}", $path_suffix),
                    $body,
                ))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                response.status()
            );
        }
    };
}

v1_msg_post_test!(
    v1_send,
    "send",
    json!({"to": "test@example.com", "subject": "Test", "body": "Hello"})
);
v1_msg_post_test!(v1_reply, "reply", json!({"body": "Reply text"}));
v1_msg_post_test!(v1_reply_all, "reply-all", json!({"body": "Reply all text"}));
v1_msg_post_test!(v1_forward, "forward", json!({"to": "fwd@example.com"}));
v1_msg_post_test!(
    v1_forward_eml,
    "forward-eml",
    json!({"to": "fwd@example.com"})
);
v1_msg_post_test!(
    v1_redirect_missing_message,
    "redirect",
    json!({"to": ["redirect@example.com"], "confirmed_provider_write": true})
);
v1_msg_post_test!(v1_imap_mark_read, "imap-mark-read", json!({}));
v1_msg_post_test!(v1_imap_delete, "imap-delete", json!({}));
v1_msg_post_test!(v1_translate, "translate", json!({"target_language": "es"}));
v1_msg_post_test!(v1_ai_reply, "ai-reply", json!({"prompt": "Reply to this"}));
v1_msg_post_test!(
    v1_ai_reply_variants,
    "ai-reply-variants",
    json!({"prompt": "Reply variants"})
);
v1_msg_post_test!(v1_extract_tasks, "extract-tasks", json!({}));
v1_msg_post_test!(v1_extract_notes, "extract-notes", json!({}));
v1_msg_post_test!(
    v1_message_analyze,
    "analyze",
    json!({"analysis_type": "sentiment"})
);

#[tokio::test]
async fn v1_message_analyze_returns_structured_ai_summary_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-analyze-summary-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("provider-analyze-summary-{suffix}"),
        "Action Required: Contract review deadline",
        "From: Ada Lovelace <ada@acme.example>\nPlease review the attached MSA and NDA by Friday. The payment risk remains open. Meeting on Monday at 10:00 with Acme Corp. Confirm approval before EOD.",
    )
    .await;
    let r = router(&context.connection_string()).await;

    let response = r
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/analyze"),
            json!({}),
        ))
        .await
        .expect("analyze response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert!(
        body["summary_contract"]["key_points"]
            .as_array()
            .expect("key points")
            .iter()
            .any(|item| item.as_str() == Some("Action Required: Contract review deadline"))
    );
    assert!(
        body["summary_contract"]["action_items"]
            .as_array()
            .expect("action items")
            .iter()
            .any(|item| {
                let item = item.as_str().unwrap_or("");
                item.contains("Please review") && item.contains("NDA")
            })
    );
    assert!(
        body["summary_contract"]["risks"]
            .as_array()
            .expect("risks")
            .iter()
            .any(|item| item.as_str().unwrap_or("").contains("payment risk"))
    );
    assert!(
        body["summary_contract"]["deadlines"]
            .as_array()
            .expect("deadlines")
            .iter()
            .any(|item| item.as_str().unwrap_or("").contains("Friday"))
    );
    assert!(
        body["summary_contract"]["event_candidates"]
            .as_array()
            .expect("event candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("Meeting on Monday"))
    );
    assert!(
        body["summary_contract"]["persona_candidates"]
            .as_array()
            .expect("persona candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("Ada Lovelace"))
    );
    assert!(
        body["summary_contract"]["organization_candidates"]
            .as_array()
            .expect("organization candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("acme.example"))
    );
    assert!(
        body["summary_contract"]["document_candidates"]
            .as_array()
            .expect("document candidates")
            .iter()
            .any(|item| item["title"].as_str().unwrap_or("").contains("MSA"))
    );
    assert!(
        body["summary_contract"]["agreement_candidates"]
            .as_array()
            .expect("agreement candidates")
            .iter()
            .any(|item| item["title"].as_str().unwrap_or("").contains("NDA"))
    );

    let metadata: Value = sqlx::query_scalar(
        "SELECT message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message metadata");
    assert_eq!(
        metadata["ai_summary_contract"], body["summary_contract"],
        "analyze response must persist the structured summary contract"
    );
}

#[tokio::test]
async fn v1_bulk_actions_mark_read_and_trash_messages_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-bulk-actions-{suffix}");
    let first_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-bulk-actions-{suffix}-1"),
        "Bulk actions first",
    )
    .await;
    let second_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-bulk-actions-{suffix}-2"),
        "Bulk actions second",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/messages/bulk-actions",
            json!({
                "action": "mark_read",
                "message_ids": [first_id, second_id]
            }),
        ))
        .await
        .expect("bulk mark-read response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["action"], "mark_read");
    assert_eq!(body["matched_count"], 2);
    assert_eq!(body["updated_count"], 2);
    assert_eq!(body["not_found"].as_array().expect("not_found").len(), 0);

    let first_workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&first_id)
    .fetch_one(&pool)
    .await
    .expect("first workflow state");
    let second_workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&second_id)
    .fetch_one(&pool)
    .await
    .expect("second workflow state");
    assert_eq!(first_workflow_state, "reviewed");
    assert_eq!(second_workflow_state, "reviewed");
    let read_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.message.read'
          AND payload->>'action' = 'mark_read'
          AND payload->>'updated_count' = '2'
          AND payload->'message_ids' ? $1
          AND payload->'message_ids' ? $2
        "#,
    )
    .bind(&first_id)
    .bind(&second_id)
    .fetch_one(&pool)
    .await
    .expect("read event count");
    assert_eq!(read_event_count, 1);

    let missing_id = format!("msg:missing-{suffix}");
    let response = r
        .oneshot(post(
            "/api/v1/communications/messages/bulk-actions",
            json!({
                "action": "trash",
                "message_ids": [first_id, missing_id]
            }),
        ))
        .await
        .expect("bulk trash response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["action"], "trash");
    assert_eq!(body["matched_count"], 1);
    assert_eq!(body["updated_count"], 1);
    assert_eq!(body["not_found"], json!([missing_id]));

    let first_local_state: String =
        sqlx::query_scalar("SELECT local_state FROM communication_messages WHERE message_id = $1")
            .bind(&first_id)
            .fetch_one(&pool)
            .await
            .expect("first local state");
    assert_eq!(first_local_state, "trash");

    let deleted_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.message.deleted'
          AND payload->>'action' = 'trash'
          AND payload->>'updated_count' = '1'
          AND payload->'message_ids' ? $1
          AND payload->'not_found' ? $2
        "#,
    )
    .bind(&first_id)
    .bind(&missing_id)
    .fetch_one(&pool)
    .await
    .expect("deleted event count");
    assert_eq!(deleted_event_count, 1);
}

#[tokio::test]
async fn v1_delete_draft() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let draft_id = format!("draft:fake-{}", uid());
    let r = router(&db).await;
    let response = r
        .oneshot(delete(&format!("/api/v1/communications/drafts/{draft_id}")))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "delete draft={}",
        response.status()
    );
}

#[tokio::test]
async fn v1_delete_message_label() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip");
        return;
    };
    let r = router(&db).await;
    let response = r
        .oneshot(delete("/api/v1/communications/messages/msg:fake/labels"))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "delete label={}",
        response.status()
    );
}

#[tokio::test]
async fn v1_imap_delete_alias_moves_message_to_local_trash_against_postgres() {
    let Some(db) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skip imap delete local trash: no DB");
        return;
    };
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let account_id = format!("acct-local-trash-api-{suffix}");
    let message_id = seed_projected_message(
        pool,
        &account_id,
        &format!("provider-local-trash-api-{suffix}"),
        "Local trash API",
    )
    .await;

    let r = router(&db).await;
    let response = r
        .clone()
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/imap-delete"),
            json!({}),
        ))
        .await
        .expect("imap delete alias");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["deleted"], true);
    assert_eq!(body["local_state"], "trash");

    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&q=Local%20trash%20API"
        )))
        .await
        .expect("active list");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 0);

    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&q=Local%20trash%20API&local_state=trash"
        )))
        .await
        .expect("trash list");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["items"][0]["local_state"], "trash");

    let response = r
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/restore"),
            json!({}),
        ))
        .await
        .expect("restore local trash");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["local_state"], "active");
}

#[tokio::test]
async fn v1_redirect_message_enqueues_outbox_redirect_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-redirect-api-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-redirect-api-{suffix}"),
        "Redirect original subject",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/redirect"),
            json!({
                "to": ["redirect@example.com"],
                "cc": ["copy@example.com"],
                "confirmed_provider_write": true
            }),
        ))
        .await
        .expect("redirect response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["transport"], "outbox");
    assert_eq!(body["status"], "queued");
    assert_eq!(body["accepted_recipients"], json!(["redirect@example.com"]));
    let outbox_id = body["outbox_id"].as_str().expect("outbox id");

    let row = sqlx::query(
        r#"
        SELECT account_id, to_recipients, cc_recipients, subject, body_text, metadata
        FROM email_outbox_tracking
        WHERE outbox_id = $1
        "#,
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("outbox row");
    assert_eq!(row.try_get::<String, _>("account_id").unwrap(), account_id);
    assert_eq!(
        row.try_get::<String, _>("subject").unwrap(),
        "Redirect original subject"
    );
    assert_eq!(
        row.try_get::<String, _>("body_text").unwrap(),
        "Body for local trash API"
    );
    assert_eq!(
        row.try_get::<Value, _>("to_recipients").unwrap(),
        json!(["redirect@example.com"])
    );
    assert_eq!(
        row.try_get::<Value, _>("cc_recipients").unwrap(),
        json!(["copy@example.com"])
    );
    let metadata = row.try_get::<Value, _>("metadata").unwrap();
    assert_eq!(metadata["redirect_of"], message_id);
    assert_eq!(metadata["redirect_mode"], "resent");
    assert_eq!(metadata["original_sender"], "sender@example.com");
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
    seed_projected_message_with_body(
        pool,
        account_id,
        provider_record_id,
        subject,
        "Body for local trash API",
    )
    .await
}

async fn seed_projected_message_with_body(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
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
                "body_text": body_text
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
