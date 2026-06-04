use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use serde_json::json;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use hermes_hub_backend::email_import::{FixtureEmailImportRequest, import_fixture_email_messages};
use hermes_hub_backend::email_sources::{FixtureEmailMessage, parse_fixture_email_messages};
use hermes_hub_backend::storage::Database;

#[test]
fn fixture_email_source_parses_account_scoped_messages() {
    let input = json!([
        {
            "provider_record_id": "gmail-msg-1",
            "subject": "Budget review",
            "from": "alice@example.com",
            "to": ["bob@example.com"],
            "sent_at": "2026-06-04T10:00:00Z",
            "body_text": "Please review the Q2 budget.",
            "source_fingerprint": "sha256:gmail-msg-1"
        }
    ])
    .to_string();

    let messages = parse_fixture_email_messages(&input).expect("parse fixture messages");

    assert_eq!(
        messages,
        vec![FixtureEmailMessage {
            provider_record_id: "gmail-msg-1".to_owned(),
            subject: "Budget review".to_owned(),
            from: "alice@example.com".to_owned(),
            to: vec!["bob@example.com".to_owned()],
            sent_at: Utc.with_ymd_and_hms(2026, 6, 4, 10, 0, 0).single(),
            body_text: "Please review the Q2 budget.".to_owned(),
            source_fingerprint: "sha256:gmail-msg-1".to_owned(),
        }]
    );
}

#[tokio::test]
async fn fixture_email_import_records_raw_messages_idempotently_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live fixture email import test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_import_{suffix}");
    let fixture_json = format!(
        r#"[{{"provider_record_id":"fixture-msg-{suffix}","subject":"Fixture import","from":"alice@example.com","to":["bob@example.com"],"sent_at":"2026-06-04T10:00:00Z","body_text":"Fixture body","source_fingerprint":"sha256:fixture-msg-{suffix}"}}]"#
    );

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Fixture Gmail",
            format!("fixture-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let first = import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(&account_id, format!("batch_{suffix}"), &fixture_json),
    )
    .await
    .expect("first import");
    let second = import_fixture_email_messages(
        &communication_store,
        &FixtureEmailImportRequest::new(
            &account_id,
            format!("batch_retry_{suffix}"),
            &fixture_json,
        ),
    )
    .await
    .expect("second import");

    assert_eq!(first.inserted_or_existing_records, 1);
    assert_eq!(second.inserted_or_existing_records, 1);

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_raw_records WHERE account_id = $1 AND provider_record_id = $2",
    )
    .bind(&account_id)
    .bind(format!("fixture-msg-{suffix}"))
    .fetch_one(&pool)
    .await
    .expect("raw record count");
    assert_eq!(count, 1);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
