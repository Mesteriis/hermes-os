use hermes_hub_backend::storage::{Database, ReadinessStatus};

#[tokio::test]
async fn database_without_url_reports_not_configured() {
    let database = Database::connect(None).await.expect("disabled database");

    let readiness = database.readiness().await;

    assert_eq!(readiness.status(), ReadinessStatus::NotConfigured);
    assert_eq!(readiness.message(), "DATABASE_URL is not configured");
}

#[tokio::test]
async fn database_without_url_reports_migrations_not_configured() {
    let database = Database::connect(None).await.expect("disabled database");

    let readiness = database.migration_readiness().await;

    assert_eq!(readiness.status(), ReadinessStatus::NotConfigured);
    assert_eq!(readiness.message(), "DATABASE_URL is not configured");
}
