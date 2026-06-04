use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use hermes_hub_backend::messages::{MessageProjectionStore, project_raw_email_message};
use hermes_hub_backend::storage::Database;

#[tokio::test]
async fn message_projection_upserts_canonical_message_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live message projection test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    let suffix = unique_suffix();
    let account_id = format!("acct_message_projection_{suffix}");
    let raw_record_id = format!("raw_message_projection_{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Projection Gmail",
            format!("projection-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                format!("provider-message-{suffix}"),
                format!("sha256:provider-message-{suffix}"),
                format!("batch_{suffix}"),
                json!({"subject":"Projected subject","from":"alice@example.com","to":["bob@example.com"],"body_text":"Projected body"}),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"fixture_email"})),
        )
        .await
        .expect("record raw message");

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    assert_eq!(projected.account_id, account_id);
    assert_eq!(
        projected.provider_record_id,
        format!("provider-message-{suffix}")
    );
    assert_eq!(projected.subject, "Projected subject");
    assert_eq!(projected.sender, "alice@example.com");
    assert_eq!(projected.recipients, vec!["bob@example.com".to_owned()]);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
