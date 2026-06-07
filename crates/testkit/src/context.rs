use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::containers::postgres::PostgresContainer;

/// Isolated test environment with a fresh PostgreSQL container and migrated database.
///
/// Each `TestContext` creates its own container and unique database, guaranteeing
/// complete isolation between tests. No shared state, no cleanup races.
///
/// # Usage
///
/// ```ignore
/// #[tokio::test]
/// async fn my_test() {
///     let ctx = TestContext::new().await;
///     let pool = ctx.pool();
///     // ... use pool ...
///     // Container and database are cleaned up on ctx.drop()
/// }
/// ```
pub struct TestContext {
    _container: PostgresContainer,
    db_name: String,
    pool: PgPool,
}

impl TestContext {
    /// Create a new isolated test environment.
    ///
    /// 1. Starts a pgvector container
    /// 2. Creates a unique database (uuid-based name)
    /// 3. Runs all sqlx migrations
    /// 4. Returns a ready-to-use pool
    pub async fn new() -> Self {
        let container = PostgresContainer::start().await;
        let db_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', "_"));

        let pool = container.create_database(&db_name).await;

        Self {
            _container: container,
            db_name,
            pool,
        }
    }

    /// The connection pool to the isolated test database.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// The full connection string for this test database.
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://testuser:testpass@127.0.0.1:{}/{}",
            self._container.host_port(),
            self.db_name
        )
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Database is cleaned up when the container is dropped
        // The container will be removed automatically
    }
}
