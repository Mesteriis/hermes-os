use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use hermes_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::domains::communications::storage::{
    AttachmentSafetyScanStatus, CommunicationAttachmentDisposition, CommunicationStorageError,
    CommunicationStorageStore, LocalCommunicationBlobStore, NewCommunicationAttachment,
    NewCommunicationBlob,
};
use hermes_hub_backend::platform::storage::Database;
use serde_json::json;

#[tokio::test]
async fn local_mail_blob_store_writes_content_addressed_blob_under_root() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let store = LocalCommunicationBlobStore::new(temp_dir.path());
    let first = store
        .put_blob(b"raw message bytes")
        .await
        .expect("write first blob");
    let second = store
        .put_blob(b"raw message bytes")
        .await
        .expect("write same blob again");

    assert_eq!(first, second);
    assert_eq!(first.storage_kind, "local_fs");
    assert_eq!(first.size_bytes, 17);
    assert!(first.sha256.starts_with("sha256:"));
    assert!(!first.storage_path.starts_with('/'));
    assert!(!first.storage_path.contains(".."));
    assert!(temp_dir.path().join(&first.storage_path).is_file());
}

#[tokio::test]
async fn mail_storage_records_attachment_metadata_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let mail_store = CommunicationStorageStore::new(pool.clone());
    let blob_root = tempfile::tempdir().expect("blob root");
    let local_blob_store = LocalCommunicationBlobStore::new(blob_root.path());
    let suffix = unique_suffix();
    let account_id = format!("acct_mail_storage_{suffix}");
    let provider_record_id = format!("mail-storage-message-{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Icloud,
            "Mail storage account",
            format!("mail-storage-{suffix}@example.invalid"),
        ))
        .await
        .expect("provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-mail-storage-{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:raw-mail-storage-{suffix}"),
                format!("batch-mail-storage-{suffix}"),
                json!({
                    "subject": "Attachment storage",
                    "from": "sender@example.invalid",
                    "to": ["recipient@example.invalid"],
                    "body_text": "See attached file."
                }),
            )
            .provenance(json!({"source": "mail_storage_test"})),
        )
        .await
        .expect("raw record");
    let message = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let local_blob = local_blob_store
        .put_blob(b"pdf contents")
        .await
        .expect("write local attachment blob");
    let blob = mail_store
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&local_blob).content_type("application/pdf"),
        )
        .await
        .expect("upsert blob");
    let attachment = mail_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message.message_id,
                &raw.raw_record_id,
                &blob.blob_id,
                "part-1",
                "application/pdf",
                local_blob.size_bytes,
                &blob.sha256,
            )
            .filename("invoice.pdf")
            .disposition(CommunicationAttachmentDisposition::Attachment),
        )
        .await
        .expect("upsert attachment");

    assert_eq!(attachment.message_id, message.message_id);
    assert_eq!(attachment.raw_record_id, raw.raw_record_id);
    assert_eq!(attachment.blob_id, blob.blob_id);
    assert_eq!(attachment.filename.as_deref(), Some("invoice.pdf"));
    assert_eq!(attachment.content_type, "application/pdf");
    assert_eq!(attachment.size_bytes, 12);
    assert_eq!(
        attachment.disposition,
        CommunicationAttachmentDisposition::Attachment
    );
    assert_eq!(
        attachment.scan_status,
        AttachmentSafetyScanStatus::NotScanned
    );
    assert!(attachment.scan_engine.is_none());
    assert!(attachment.scan_checked_at.is_none());
    assert!(attachment.scan_summary.is_none());
    assert_eq!(attachment.scan_metadata, json!({}));

    let attachment_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_attachments WHERE message_id = $1",
    )
    .bind(&message.message_id)
    .fetch_one(&pool)
    .await
    .expect("attachment count");
    assert_eq!(attachment_count, 1);
}

#[tokio::test]
async fn mail_blob_metadata_rejects_unsafe_storage_path_before_database_write() {
    let store =
        CommunicationStorageStore::new(sqlx::PgPool::connect_lazy("postgres://unused").unwrap());
    let error = store
        .upsert_blob(&NewCommunicationBlob::new(
            "local_fs",
            "../outside.blob",
            "sha256:unsafe",
            1,
        ))
        .await
        .expect_err("unsafe path must fail");

    assert!(
        matches!(error, CommunicationStorageError::UnsafeStoragePath(ref path) if path == "../outside.blob"),
        "expected UnsafeStoragePath, got {error:?}"
    );
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
