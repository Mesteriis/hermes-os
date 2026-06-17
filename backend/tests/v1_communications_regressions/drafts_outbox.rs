use axum::http::StatusCode;
use chrono::{Duration, Utc};
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use serde_json::json;
use testkit::context::TestContext;
use tower::ServiceExt;

use super::support::{delete, get, post, post_with_actor, response_json, router, uid};

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

    let router = router(&context.connection_string()).await;
    let draft_id = format!("draft-autosave-{suffix}");
    let response = router
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

    let response = router
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

    let response = router
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

    let router = router(&context.connection_string()).await;
    for index in 0..2 {
        let response = router
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

    let response = router
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

    let response = router
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

    let router = router(&context.connection_string()).await;
    let scheduled_send_at = Utc::now() + Duration::hours(1);
    let send = router
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

    let list = router
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

    let undo = router
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

    let canceled = router
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
