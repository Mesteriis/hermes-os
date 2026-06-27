use sqlx::postgres::PgPool;
use tokio::sync::{Mutex, OnceCell};
use uuid::Uuid;

use crate::containers::nats::NatsContainer;
use crate::containers::postgres::PostgresContainer;
use crate::vault::{TestVault, new_test_vault};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::storage::Database;
use std::path::Path;

static POSTGRES_CONTAINER: OnceCell<PostgresContainer> = OnceCell::const_new();
static NATS_CONTAINER: OnceCell<NatsContainer> = OnceCell::const_new();
static DATABASE_SETUP_LOCK: Mutex<()> = Mutex::const_new(());

/// Isolated test environment with a fresh migrated database.
///
/// Each `TestContext` creates its own unique database on the PostgreSQL
/// container owned by the current test session. `make backend-test` starts one
/// pgvector container and one NATS container before nextest, then removes them
/// after the session.
///
/// # Usage
///
/// ```ignore
/// #[tokio::test]
/// async fn my_test() {
///     let ctx = TestContext::new().await;
///     let pool = ctx.pool();
///     // ... use pool ...
///     // The pool is dropped with the context.
/// }
/// ```
pub struct TestContext {
    container: &'static PostgresContainer,
    db_name: String,
    vault: TestVault,
    pool: PgPool,
}

impl TestContext {
    /// Create a new isolated test environment.
    ///
    /// 1. Reuses the pgvector container for this test session
    /// 2. Creates a unique database (uuid-based name)
    /// 3. Runs all sqlx migrations
    /// 4. Returns a ready-to-use pool
    pub async fn new() -> Self {
        let container = POSTGRES_CONTAINER
            .get_or_init(PostgresContainer::start)
            .await;
        let db_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', "_"));

        let _setup_guard = DATABASE_SETUP_LOCK.lock().await;
        let pool = container.create_database(&db_name).await;
        let vault = new_test_vault();

        Self {
            container,
            db_name,
            vault,
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
            self.container.host_port(),
            self.db_name
        )
    }

    pub fn vault_home(&self) -> &Path {
        self.vault.vault_home()
    }

    pub fn dev_key_path(&self) -> &Path {
        self.vault.dev_key_path()
    }

    pub fn vault_database_path(&self) -> std::path::PathBuf {
        self.vault.vault_database_path()
    }

    pub fn app_config(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault
            .apply_to_config(AppConfig::test_with_api_secret_and_database_url(
                api_secret,
                self.connection_string(),
            ))
    }

    pub async fn nats_server_url(&self) -> String {
        NATS_CONTAINER
            .get_or_init(NatsContainer::start)
            .await
            .server_url()
    }

    pub async fn app_config_with_nats(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault.apply_to_config(
            AppConfig::test_with_api_secret_and_database_url(api_secret, self.connection_string())
                .with_test_pairs([("HERMES_NATS_SERVER_URL", self.nats_server_url().await)])
                .expect("test NATS config must be valid"),
        )
    }

    pub fn app_config_without_database(&self, api_secret: impl Into<String>) -> AppConfig {
        self.vault
            .apply_to_config(AppConfig::test_with_api_secret(api_secret))
    }

    /// Database runtime wrapper backed by this context's migrated pool.
    pub fn database(&self) -> Database {
        Database::from_test_pool(self.pool.clone(), self.connection_string())
    }
}
