use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use chrono::{TimeZone, Utc};
use serde_json::json;
use sqlx::Row;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use hermes_hub_backend::email_sync::{EmailSyncBatch, FetchedEmailMessage};
use hermes_hub_backend::email_sync_pipeline::project_email_sync_batch_with_mail_blobs;
use hermes_hub_backend::mail_storage::LocalMailBlobStore;
use hermes_hub_backend::storage::Database;

#[tokio::test]
async fn email_sync_pipeline_records_raw_blob_and_projects_message_persons_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live email sync pipeline test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_sync_pipeline_{suffix}");
    let provider_record_id = format!("sync-pipeline-message-{suffix}");
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalMailBlobStore::new(blob_root.path());

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Sync pipeline IMAP",
            format!("sync-pipeline-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");

    let batch = EmailSyncBatch {
        provider_kind: EmailProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 88})),
        messages: vec![FetchedEmailMessage {
            provider_record_id: provider_record_id.clone(),
            source_fingerprint: format!("sha256:sync-pipeline-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_000_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 88,
                "raw_rfc822_base64": "U3ViamVjdDogU3luYyBQaXBlbGluZQ0KRnJvbTogU2VuZGVyIDxzZW5kZXJAZXhhbXBsZS5pbnZhbGlkPg0KVG86IFJlY2lwaWVudCA8cmVjaXBpZW50QGV4YW1wbGUuaW52YWxpZD4NCkNvbnRlbnQtVHlwZTogdGV4dC9wbGFpbjsgY2hhcnNldD11dGYtOA0KDQpDYWNoZWQgbWVzc2FnZSBib2R5Lg=="
            }),
        }],
    };

    let report = project_email_sync_batch_with_mail_blobs(
        pool.clone(),
        &blob_store,
        &account_id,
        format!("sync-pipeline-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("project email sync batch");

    assert_eq!(report.imported_records, 1);
    assert_eq!(report.raw_blobs_upserted, 1);
    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.attachment_blobs_upserted, 0);
    assert_eq!(report.attachments_extracted, 0);
    assert_eq!(report.attachments_not_scanned, 0);
    assert_eq!(report.upserted_persons, 2);

    let projected = sqlx::query(
        r#"
        SELECT subject, sender, recipients, body_text
        FROM communication_messages
        WHERE account_id = $1
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("projected message");
    let subject: String = projected.try_get("subject").expect("subject");
    let sender: String = projected.try_get("sender").expect("sender");
    let recipients: serde_json::Value = projected.try_get("recipients").expect("recipients");
    let body_text: String = projected.try_get("body_text").expect("body_text");
    assert_eq!(subject, "Sync Pipeline");
    assert_eq!(sender, "Sender <sender@example.invalid>");
    assert_eq!(body_text, "Cached message body.");
    assert_eq!(recipients, json!(["Recipient <recipient@example.invalid>"]));
}

#[tokio::test]
async fn email_sync_pipeline_extracts_attachment_metadata_with_initial_scan_status() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live email sync pipeline test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_sync_attachment_{suffix}");
    let provider_record_id = format!("sync-attachment-message-{suffix}");
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalMailBlobStore::new(blob_root.path());

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Imap,
            "Sync attachment IMAP",
            format!("sync-attachment-{suffix}@example.net"),
        ))
        .await
        .expect("store provider account");

    let raw_rfc822 = concat!(
        "Subject: Attachment Pipeline\r\n",
        "From: Sender <sender@example.invalid>\r\n",
        "To: Recipient <recipient@example.invalid>\r\n",
        "Content-Type: multipart/mixed; boundary=\"hermes-boundary\"\r\n",
        "\r\n",
        "--hermes-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "See attached cache fixture.\r\n",
        "--hermes-boundary\r\n",
        "Content-Type: text/plain; name=\"invoice.txt\"\r\n",
        "Content-Disposition: attachment; filename=\"invoice.txt\"\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "YXR0YWNobWVudCBieXRlcw==\r\n",
        "--hermes-boundary--\r\n"
    );
    let raw_rfc822_base64 = base64::engine::general_purpose::STANDARD.encode(raw_rfc822);
    let batch = EmailSyncBatch {
        provider_kind: EmailProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 89})),
        messages: vec![FetchedEmailMessage {
            provider_record_id: provider_record_id.clone(),
            source_fingerprint: format!("sha256:sync-attachment-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_100_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 89,
                "raw_rfc822_base64": raw_rfc822_base64
            }),
        }],
    };

    let report = project_email_sync_batch_with_mail_blobs(
        pool.clone(),
        &blob_store,
        &account_id,
        format!("sync-attachment-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("project email sync batch");

    assert_eq!(report.imported_records, 1);
    assert_eq!(report.raw_blobs_upserted, 1);
    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.attachment_blobs_upserted, 1);
    assert_eq!(report.attachments_extracted, 1);
    assert_eq!(report.attachments_not_scanned, 1);

    let attachment = sqlx::query(
        r#"
        SELECT
            a.filename,
            a.content_type,
            a.size_bytes,
            a.sha256,
            a.disposition,
            a.scan_status,
            a.scan_engine,
            a.scan_checked_at,
            a.scan_summary,
            a.scan_metadata,
            b.storage_kind,
            b.storage_path
        FROM communication_attachments a
        JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
        JOIN communication_messages m ON m.message_id = a.message_id
        WHERE m.account_id = $1
          AND m.provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("projected attachment metadata");

    let filename: Option<String> = attachment.try_get("filename").expect("filename");
    let content_type: String = attachment.try_get("content_type").expect("content_type");
    let size_bytes: i64 = attachment.try_get("size_bytes").expect("size_bytes");
    let sha256: String = attachment.try_get("sha256").expect("sha256");
    let disposition: String = attachment.try_get("disposition").expect("disposition");
    let scan_status: String = attachment.try_get("scan_status").expect("scan_status");
    let scan_engine: Option<String> = attachment.try_get("scan_engine").expect("scan_engine");
    let scan_checked_at: Option<chrono::DateTime<Utc>> = attachment
        .try_get("scan_checked_at")
        .expect("scan_checked_at");
    let scan_summary: Option<String> = attachment.try_get("scan_summary").expect("scan_summary");
    let scan_metadata: serde_json::Value =
        attachment.try_get("scan_metadata").expect("scan_metadata");
    let storage_kind: String = attachment.try_get("storage_kind").expect("storage_kind");
    let storage_path: String = attachment.try_get("storage_path").expect("storage_path");

    assert_eq!(filename.as_deref(), Some("invoice.txt"));
    assert_eq!(content_type, "text/plain");
    assert_eq!(size_bytes, 16);
    assert!(sha256.starts_with("sha256:"));
    assert_eq!(disposition, "attachment");
    assert_eq!(scan_status, "not_scanned");
    assert!(scan_engine.is_none());
    assert!(scan_checked_at.is_none());
    assert!(scan_summary.is_none());
    assert_eq!(scan_metadata, json!({}));
    assert_eq!(storage_kind, "local_fs");
    assert!(blob_root.path().join(storage_path).is_file());
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
