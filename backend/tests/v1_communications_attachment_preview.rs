use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::domains::communications::storage::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus, CommunicationAttachmentDisposition,
    CommunicationStorageStore, LocalCommunicationBlobStore, NewCommunicationAttachment,
    NewCommunicationBlob,
};
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::workflows::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use testkit::context::TestContext;

const T: &str = "v1comms-attachment-preview-test-token";

#[tokio::test]
async fn v1_attachment_preview_reads_bounded_local_text_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "notes.txt",
        "text/plain",
        AttachmentSafetyScanStatus::NotScanned,
        b"First line\nSecond line\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["filename"], "notes.txt");
    assert_eq!(body["content_type"], "text/plain");
    assert_eq!(body["scan_status"], "not_scanned");
    assert_eq!(body["preview_kind"], "text");
    assert_eq!(body["text"], "First line\nSecond line\n");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["byte_count"], 23);
    assert_eq!(body["max_preview_bytes"], 65536);
}

#[tokio::test]
async fn v1_attachment_preview_reads_bounded_local_image_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "pixel.png",
        "image/png",
        AttachmentSafetyScanStatus::NotScanned,
        b"\x89PNG\r\n\x1a\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment image preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["filename"], "pixel.png");
    assert_eq!(body["content_type"], "image/png");
    assert_eq!(body["preview_kind"], "image");
    assert_eq!(body["text"], "");
    assert_eq!(body["data_url"], "data:image/png;base64,iVBORw0KGgo=");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["byte_count"], 8);
}

#[tokio::test]
async fn v1_attachment_preview_reads_bounded_local_pdf_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "spec.pdf",
        "application/pdf",
        AttachmentSafetyScanStatus::NotScanned,
        b"%PDF-1.4\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment pdf preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["filename"], "spec.pdf");
    assert_eq!(body["content_type"], "application/pdf");
    assert_eq!(body["preview_kind"], "pdf");
    assert_eq!(body["text"], "");
    assert_eq!(body["data_url"], "data:application/pdf;base64,JVBERi0xLjQK");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["byte_count"], 9);
    assert_eq!(body["max_preview_bytes"], 16777216);
}

#[tokio::test]
async fn v1_attachment_preview_rejects_malicious_attachment_metadata() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "danger.txt",
        "text/plain",
        AttachmentSafetyScanStatus::Malicious,
        b"This text must not be exposed through preview.",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment preview rejection response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert_eq!(
        body["message"],
        "attachment preview is blocked by attachment scan status"
    );
}

struct SeededAttachment {
    attachment_id: String,
    message_id: String,
}

async fn seed_text_attachment(
    pool: sqlx::PgPool,
    filename: &str,
    content_type: &str,
    scan_status: AttachmentSafetyScanStatus,
    bytes: &[u8],
) -> SeededAttachment {
    let suffix = uid();
    let account_id = format!("acct-attachment-preview-{suffix}");
    let provider_record_id = format!("provider-attachment-preview-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Attachment Preview Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            &account_id,
            "email_message",
            &provider_record_id,
            format!("sha256:{:0>64}", "f"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Attachment preview",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Preview the attached text metadata safely."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;

    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = local_blob_store
        .put_blob(bytes)
        .await
        .expect("write text blob");
    let blob = storage_store
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type(content_type))
        .await
        .expect("store text blob metadata");
    let attachment = storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                format!("part-{filename}"),
                content_type,
                local_blob.size_bytes,
                local_blob.sha256,
            )
            .filename(filename)
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: scan_status,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store text attachment");

    SeededAttachment {
        attachment_id: attachment.attachment_id,
        message_id,
    }
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-hermes-secret", T)
        .body(Body::empty())
        .expect("request")
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
