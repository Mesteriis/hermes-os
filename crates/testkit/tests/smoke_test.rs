//! Smoke test for Testcontainers infrastructure in crates/testkit.
//!
//! Verifies that:
//! - AC1: Testcontainers PostgreSQL контейнер поднимается и проходит health check
//! - Migration runner works against the container
//! - Entity factories can create records
//!
//! This is a characterization test — it captures CURRENT behavior without
//! modifying it. Any change to the infrastructure will cause these tests to
//! fail, alerting the developer to review compatibility.

use testkit::context::TestContext;

/// AC1: Testcontainers PostgreSQL container starts and passes health check.
#[tokio::test]
async fn test_context_creates_isolated_database() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool();

    // Verify basic query works (container is reachable, DB is responsive)
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(pool)
        .await
        .expect("basic query against test database must succeed");
    assert_eq!(result, 1, "pgvector container must respond to queries");
}

/// Verify migrations were applied successfully.
#[tokio::test]
async fn test_context_runs_migrations() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool();

    // Check that _sqlx_migrations table exists (proof migrations ran)
    let migration_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true",
    )
    .fetch_one(pool)
    .await
    .expect("migrations table must exist and be queryable");
    assert!(
        migration_count > 0,
        "at least one migration must have been applied (got {migration_count})"
    );
}

/// Verify each test gets a unique, isolated database.
#[tokio::test]
async fn test_context_databases_are_isolated() {
    let ctx_a = TestContext::new().await;
    let ctx_b = TestContext::new().await;

    let a_conn = ctx_a.connection_string();
    let b_conn = ctx_b.connection_string();

    assert_ne!(
        a_conn, b_conn,
        "each TestContext must produce a unique database connection string"
    );
}

/// Verify the ContactFactory works against a real container.
#[tokio::test]
async fn testkit_contact_factory_creates_person() {
    let ctx = TestContext::new().await;
    let factory = testkit::factories::contact::ContactFactory::new(ctx.pool());

    let person_id = factory
        .with_name("Characterization Test Person")
        .with_email("char-test@example.com")
        .create()
        .await
        .expect("ContactFactory must create a person against the test container");

    assert!(
        !person_id.is_empty(),
        "ContactFactory must return a non-empty person ID"
    );

    // Verify the person actually exists in the database
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM persons WHERE person_id = $1)",
    )
    .bind(&person_id)
    .fetch_one(ctx.pool())
    .await
    .expect("query must succeed");
    assert!(exists, "person created by ContactFactory must exist in DB");
}

/// Verify the EmailFactory works against a real container.
#[tokio::test]
async fn testkit_email_factory_creates_email() {
    let ctx = TestContext::new().await;
    let factory = testkit::factories::email::EmailFactory::new(ctx.pool());

    let (_account, raw) = factory
        .with_subject("Characterization Test Email")
        .with_from("sender@example.com")
        .with_body("This is a characterization test email body.")
        .create()
        .await
        .expect("EmailFactory must create an email record against the test container");

    assert!(
        !raw.raw_record_id.is_empty(),
        "EmailFactory must return a non-empty raw_record_id"
    );
}
