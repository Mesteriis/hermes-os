use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
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
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus, MailAttachmentDisposition,
    MailStorageStore, NewMailAttachment, NewMailBlob,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use testkit::context::TestContext;

const LOCAL_API_TOKEN: &str = "v1comms-attachment-translation-test-token";

#[tokio::test]
async fn v1_attachment_translation_uses_provided_extracted_text_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "Hola equipo, adjunto el contrato para revisión."
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["filename"], "contrato.txt");
    assert_eq!(body["original_language"], "es");
    assert_eq!(body["translated"], false);
    assert_eq!(body["target"], "en");
    assert_eq!(body["text"], Value::Null);
    assert_eq!(body["model"], Value::Null);
    assert_eq!(body["reason"], "translation runtime unavailable");
    assert_eq!(body["source"], "caller_provided_extracted_text");
}

#[tokio::test]
async fn v1_attachment_translation_rejects_empty_source_text_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_message_with_attachment(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(post(
            &format!(
                "/api/v1/communications/attachments/{}/translate",
                seeded.attachment_id
            ),
            json!({
                "target_language": "en",
                "source_text": "   "
            }),
        ))
        .await
        .expect("translation response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

struct SeededAttachment {
    attachment_id: String,
    message_id: String,
}

async fn seed_message_with_attachment(pool: sqlx::PgPool) -> SeededAttachment {
    let suffix = uid();
    let account_id = format!("acct-attachment-translation-{suffix}");
    let provider_record_id = format!("provider-attachment-translation-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = MailStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Attachment Translation Gmail",
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
            format!("sha256:{:0>64}", "c"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Contrato",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Please review the attached contract."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;
    let sha256 = format!("sha256:{:0>64}", "d");
    let blob = storage_store
        .upsert_blob(
            &NewMailBlob::new(
                "local_fs",
                format!("attachments/{provider_record_id}/contrato.txt"),
                &sha256,
                512,
            )
            .content_type("text/plain"),
        )
        .await
        .expect("store blob");
    let attachment = storage_store
        .upsert_attachment(
            &NewMailAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                "part-contrato",
                "text/plain",
                512,
                sha256,
            )
            .filename("contrato.txt")
            .disposition(MailAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: AttachmentSafetyScanStatus::NotScanned,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store attachment");

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
        AppConfig::from_pairs([
            ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
            ("DATABASE_URL", database_url),
        ])
        .expect("config"),
        database,
    )
}

fn post(uri: &str, value: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-hermes-secret", LOCAL_API_TOKEN)
        .body(Body::from(value.to_string()))
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
