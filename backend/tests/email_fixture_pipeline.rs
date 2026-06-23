use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use serde_json::json;

use hermes_hub_backend::domains::communications::core::EmailProviderKind;
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::workflows::email_fixture_pipeline::{
    EmailFixturePipelineRequest, project_fixture_email_messages,
};

#[tokio::test]
async fn fixture_email_pipeline_imports_projects_persons_and_graph_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

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

    let accepted_signal_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'signal.accepted.mail.message'
          AND source ->> 'account_id' = $1
        "#,
    )
    .bind(&account_id)
    .fetch_one(test_context.pool())
    .await
    .expect("accepted mail signal count");
    assert_eq!(accepted_signal_count, 1);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
