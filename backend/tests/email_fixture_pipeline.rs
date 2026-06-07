use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use hermes_hub_backend::domains::mail::core::EmailProviderKind;
use hermes_hub_backend::domains::mail::fixtures::pipeline::{
    EmailFixturePipelineRequest, project_fixture_email_messages,
};
use hermes_hub_backend::platform::storage::Database;

#[tokio::test]
async fn fixture_email_pipeline_imports_projects_persons_and_graph_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live fixture email pipeline test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("acct_fixture_pipeline_{suffix}");
    let fixture_json = json!([
        {
            "provider_record_id": format!("fixture-pipeline-{suffix}"),
            "subject": "Pipeline import",
            "from": "sender@example.invalid",
            "to": ["recipient@example.invalid"],
            "sent_at": "2026-06-04T10:00:00Z",
            "body_text": "Pipeline body",
            "source_fingerprint": format!("sha256:pipeline-{suffix}")
        }
    ])
    .to_string();

    let report = project_fixture_email_messages(
        pool,
        &EmailFixturePipelineRequest::new(
            &account_id,
            "iCloud fixture pipeline",
            "redacted@example.invalid",
            EmailProviderKind::Icloud,
            format!("batch_pipeline_{suffix}"),
            fixture_json,
        ),
    )
    .await
    .expect("project fixture pipeline");

    assert_eq!(report.imported_records, 1);
    assert_eq!(report.projected_messages, 1);
    assert_eq!(report.upserted_persons, 2);
    assert!(!report.graph_summary.is_empty);
    assert!(report.total_graph_nodes >= 4);
    assert!(report.total_graph_edges >= 3);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
