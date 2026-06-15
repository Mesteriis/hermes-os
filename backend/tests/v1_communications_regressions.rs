use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
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

const T: &str = "v1comms-regression-test-token";

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

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
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
async fn v1_post_draft_allows_empty_subject_for_autosave_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-draft-autosave-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Draft Autosave IMAP",
            format!("draft-autosave-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let r = router(&context.connection_string()).await;
    let draft_id = format!("draft-autosave-{suffix}");
    let response = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/drafts",
            json!({
                "draft_id": draft_id,
                "account_id": account_id,
                "to_recipients": [],
                "cc_recipients": [],
                "bcc_recipients": [],
                "subject": "",
                "body_text": "Body typed before subject",
                "body_html": null,
                "metadata": {"compose_mode": "compose"}
            }),
        ))
        .await
        .expect("draft autosave response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["draft_id"], draft_id);
    assert_eq!(body["subject"], "");
    assert_eq!(body["body_text"], "Body typed before subject");

    let created_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.created'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft created event count");
    assert_eq!(created_event_count, 1);

    let response = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/drafts",
            json!({
                "draft_id": draft_id,
                "account_id": account_id,
                "to_recipients": ["recipient@example.com"],
                "subject": "",
                "body_text": "Updated autosave body",
                "body_html": "<p>Updated autosave body</p>",
                "metadata": {"compose_mode": "compose"}
            }),
        ))
        .await
        .expect("draft autosave update response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["draft_id"], draft_id);
    assert_eq!(body["body_text"], "Updated autosave body");

    let updated_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.updated'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND payload->>'has_body_html' = 'true'
          AND payload->>'to_recipient_count' = '1'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft updated event count");
    assert_eq!(updated_event_count, 1);

    let response = r
        .oneshot(delete(&format!("/api/v1/communications/drafts/{draft_id}")))
        .await
        .expect("draft delete response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["deleted"], true);

    let deleted_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.deleted'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft deleted event count");
    assert_eq!(deleted_event_count, 1);
}

#[tokio::test]
async fn v1_drafts_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-draft-page-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Draft Pagination IMAP",
            format!("draft-page-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let r = router(&context.connection_string()).await;
    for index in 0..2 {
        let response = r
            .clone()
            .oneshot(post(
                "/api/v1/communications/drafts",
                json!({
                    "draft_id": format!("draft-page-{suffix}-{index}"),
                    "account_id": account_id,
                    "to_recipients": ["recipient@example.com"],
                    "subject": format!("Paged draft {index}"),
                    "body_text": format!("Draft body {index}"),
                    "metadata": {"compose_mode": "compose"}
                }),
            ))
            .await
            .expect("draft create response");
        assert_eq!(response.status(), StatusCode::OK);
    }

    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/drafts?account_id={account_id}&limit=1"
        )))
        .await
        .expect("draft list first page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = r
        .oneshot(get(&format!(
            "/api/v1/communications/drafts?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("draft list second page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}

#[tokio::test]
async fn v1_subscriptions_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-sub-page-{suffix}");
    for index in 0..3 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sub-a-{suffix}-{index}"),
            "Weekly digest",
            "newsletter-a@example.com",
            "Newsletter body with unsubscribe link",
        )
        .await;
    }
    for index in 0..2 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sub-b-{suffix}-{index}"),
            "Product newsletter",
            "newsletter-b@example.com",
            "Newsletter body with manage preferences link",
        )
        .await;
    }

    let r = router(&context.connection_string()).await;
    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/subscriptions?account_id={account_id}&limit=1"
        )))
        .await
        .expect("subscriptions first page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = r
        .oneshot(get(&format!(
            "/api/v1/communications/subscriptions?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("subscriptions second page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}

#[tokio::test]
async fn v1_top_senders_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-senders-page-{suffix}");
    for index in 0..3 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sender-a-{suffix}-{index}"),
            "Message from A",
            "sender-a@example.com",
            "Regular mail body",
        )
        .await;
    }
    for index in 0..2 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sender-b-{suffix}-{index}"),
            "Message from B",
            "sender-b@example.com",
            "Regular mail body",
        )
        .await;
    }

    let r = router(&context.connection_string()).await;
    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/analytics/senders?account_id={account_id}&limit=1"
        )))
        .await
        .expect("top senders first page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = r
        .oneshot(get(&format!(
            "/api/v1/communications/analytics/senders?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("top senders second page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}

#[tokio::test]
async fn v1_send_schedules_outbox_message_and_allows_undo_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-outbox-api-{suffix}");
    CommunicationIngestionStore::new(pool)
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Outbox API IMAP",
            format!("outbox-api-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let r = router(&context.connection_string()).await;
    let scheduled_send_at = Utc::now() + Duration::hours(1);
    let send = r
        .clone()
        .oneshot(post_with_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Scheduled outbox",
                "body_text": "This should be queued, not sent immediately.",
                "scheduled_send_at": scheduled_send_at,
                "undo_send_seconds": 30,
                "confirmed_provider_write": true
            }),
        ))
        .await
        .expect("scheduled send response");

    assert_eq!(send.status(), StatusCode::OK);
    let send_body = response_json(send).await;
    assert_eq!(send_body["transport"], "outbox");
    assert_eq!(send_body["status"], "scheduled");
    let outbox_id = send_body["outbox_id"].as_str().expect("outbox id");
    assert!(!outbox_id.trim().is_empty());

    let list = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/outbox?account_id={account_id}&status=scheduled"
        )))
        .await
        .expect("outbox list response");
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = response_json(list).await;
    let items = list_body["items"].as_array().expect("outbox items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["outbox_id"], outbox_id);
    assert_eq!(items[0]["subject"], "Scheduled outbox");

    let undo = r
        .clone()
        .oneshot(post(
            &format!("/api/v1/communications/outbox/{outbox_id}/undo"),
            json!({}),
        ))
        .await
        .expect("outbox undo response");
    assert_eq!(undo.status(), StatusCode::OK);
    let undo_body = response_json(undo).await;
    assert_eq!(undo_body["outbox_id"], outbox_id);
    assert_eq!(undo_body["status"], "canceled");

    let canceled = r
        .oneshot(get(&format!(
            "/api/v1/communications/outbox?account_id={account_id}&status=canceled"
        )))
        .await
        .expect("canceled outbox list response");
    assert_eq!(canceled.status(), StatusCode::OK);
    let canceled_body = response_json(canceled).await;
    assert_eq!(
        canceled_body["items"]
            .as_array()
            .expect("canceled outbox items")
            .len(),
        1
    );
}

#[tokio::test]
async fn v1_messages_list_uses_cursor_pagination_without_duplicates_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-cursor-api-{suffix}");
    let mut seeded_message_ids = Vec::new();

    for index in 0..3 {
        let message_id = seed_projected_message(
            pool.clone(),
            &account_id,
            &format!("provider-cursor-api-{suffix}-{index}"),
            &format!("Cursor page subject {suffix} {index}"),
        )
        .await;
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET occurred_at = now() - ($2::int * interval '1 minute'),
                projected_at = now() - ($2::int * interval '1 minute')
            WHERE message_id = $1
            "#,
        )
        .bind(&message_id)
        .bind(index)
        .execute(&pool)
        .await
        .expect("set deterministic message ordering");
        seeded_message_ids.push(message_id);
    }

    let r = router(&context.connection_string()).await;
    let first = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&limit=2"
        )))
        .await
        .expect("first cursor page");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    let first_items = first_body["items"].as_array().expect("first items");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_body["has_more"], true);
    let cursor = first_body["next_cursor"]
        .as_str()
        .expect("next cursor")
        .to_owned();
    assert!(!cursor.trim().is_empty());

    let second = r
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&limit=2&cursor={cursor}"
        )))
        .await
        .expect("second cursor page");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    let second_items = second_body["items"].as_array().expect("second items");
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_body["has_more"], false);
    assert!(second_body["next_cursor"].is_null());

    let returned_ids = first_items
        .iter()
        .chain(second_items.iter())
        .map(|item| item["message_id"].as_str().expect("message id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_ids.len(), 3);
    for message_id in seeded_message_ids {
        assert!(returned_ids.contains(&message_id), "missing {message_id}");
    }
}

#[tokio::test]
async fn v1_threads_list_uses_cursor_pagination_without_duplicates_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-cursor-api-{suffix}");

    for index in 0..3 {
        let message_id = seed_projected_message(
            pool.clone(),
            &account_id,
            &format!("provider-thread-cursor-api-{suffix}-{index}"),
            &format!("Thread Cursor Subject {suffix} {index}"),
        )
        .await;
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET occurred_at = now() - ($2::int * interval '1 minute'),
                projected_at = now() - ($2::int * interval '1 minute')
            WHERE message_id = $1
            "#,
        )
        .bind(&message_id)
        .bind(index)
        .execute(&pool)
        .await
        .expect("set deterministic thread ordering");
    }

    let r = router(&context.connection_string()).await;
    let first = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/threads?account_id={account_id}&limit=2"
        )))
        .await
        .expect("first thread cursor page");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    let first_items = first_body["items"].as_array().expect("first thread items");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_body["has_more"], true);
    let cursor = first_body["next_cursor"]
        .as_str()
        .expect("next thread cursor")
        .to_owned();
    assert!(!cursor.trim().is_empty());

    let second = r
        .oneshot(get(&format!(
            "/api/v1/communications/threads?account_id={account_id}&limit=2&cursor={cursor}"
        )))
        .await
        .expect("second thread cursor page");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    let second_items = second_body["items"]
        .as_array()
        .expect("second thread items");
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_body["has_more"], false);
    assert!(second_body["next_cursor"].is_null());

    let returned_ids = first_items
        .iter()
        .chain(second_items.iter())
        .map(|item| item["thread_id"].as_str().expect("thread id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_ids.len(), 3);
}

#[tokio::test]
async fn v1_translate_thread_returns_per_message_fallbacks_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-translate-{suffix}");
    let subject = "Thread Translation";
    let first_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-1-{suffix}"),
        subject,
        "Привет, нужна проверка договора.",
    )
    .await;
    let second_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-2-{suffix}"),
        &format!("Re: {subject}"),
        "Hello, the agreement is attached.",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            &format!(
                "/api/v1/communications/threads/translate?account_id={account_id}&subject=Thread%20Translation"
            ),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("thread translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["subject"], subject);
    assert_eq!(body["target_language"], "en");
    let items = body["items"].as_array().expect("translation items");
    assert_eq!(items.len(), 2);
    let returned_ids = items
        .iter()
        .map(|item| item["message_id"].as_str().expect("message id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert!(returned_ids.contains(&first_id));
    assert!(returned_ids.contains(&second_id));
    assert!(
        items
            .iter()
            .any(|item| item["original_language"] == "ru" && item["translated"] == false)
    );
    assert!(items.iter().all(|item| {
        item["reason"]
            .as_str()
            .map(|reason| !reason.trim().is_empty())
            .unwrap_or(false)
    }));
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
                "body_text": "Body for cursor pagination API"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
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
            "Thread Translate Gmail",
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

async fn seed_projected_message_from_sender(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
    sender: &str,
    body_text: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            EmailProviderKind::Gmail,
            "Paged Analytics Gmail",
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
                "from": sender,
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
