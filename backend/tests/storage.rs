use std::env;

use chrono::{DateTime, Utc};
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

#[tokio::test]
async fn migration_readiness_rejects_missing_latest_migration_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live migration readiness test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool");

    let migration: (i64, String, DateTime<Utc>, bool, Vec<u8>, i64) = sqlx::query_as(
        r#"
        SELECT version, description, installed_on, success, checksum, execution_time
        FROM _sqlx_migrations
        ORDER BY version DESC
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("latest sqlx migration");

    sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
        .bind(migration.0)
        .execute(pool)
        .await
        .expect("delete latest sqlx migration record");

    let readiness = database.migration_readiness().await;

    sqlx::query(
        r#"
        INSERT INTO _sqlx_migrations (
            version,
            description,
            installed_on,
            success,
            checksum,
            execution_time
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(migration.0)
    .bind(migration.1)
    .bind(migration.2)
    .bind(migration.3)
    .bind(migration.4)
    .bind(migration.5)
    .execute(pool)
    .await
    .expect("restore latest sqlx migration record");

    assert!(
        migration.0 >= 4,
        "test requires actor identity migration to exist"
    );
    assert_eq!(readiness.status(), ReadinessStatus::Unavailable);
    assert_eq!(
        readiness.message(),
        "required database migrations are incomplete"
    );
}
