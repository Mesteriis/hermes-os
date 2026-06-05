use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

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
async fn email_sync_pipeline_records_raw_blob_and_projects_message_contacts_against_postgres() {
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
    assert_eq!(report.upserted_contacts, 2);

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

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
