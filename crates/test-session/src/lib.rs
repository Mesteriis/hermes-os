//! Container lifecycle primitives for Hermes integration-test sessions.
//!
//! This crate deliberately has no dependency on application or domain crates.
//! It can start PostgreSQL, PgBouncer and NATS and expose their session-scoped endpoints;
//! This crate also owns the shared schema migration step; domain-specific
//! fixtures remain in `hermes-backend-testkit`.
//!
//! PostgreSQL remains the administrative endpoint for creating isolated test
//! databases. Runtime pools use the session PgBouncer endpoint when available,
//! so high-concurrency tests exercise the same pooling boundary as production.

pub mod containers;

#[derive(Clone)]
pub struct TestDatabase {
    pool: sqlx::PgPool,
    connection_string: String,
}

impl TestDatabase {
    pub fn new(pool: sqlx::PgPool, connection_string: impl Into<String>) -> Self {
        Self {
            pool,
            connection_string: connection_string.into(),
        }
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }
    pub fn clone_pool(&self) -> sqlx::PgPool {
        self.pool.clone()
    }
}

/// Apply the repository schema to an isolated test database.
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../backend/migrations").run(pool).await
}
