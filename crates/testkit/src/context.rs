use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::vault::{TestVault, new_test_vault};
use hermes_hub_backend::platform::config::app_config::AppConfig;
use hermes_hub_backend::platform::storage::database::Database;
use hermes_test_session::containers::nats::NatsContainer;
use hermes_test_session::containers::pgbouncer::PgbouncerContainer;
use hermes_test_session::containers::postgres::PostgresContainer;
use std::path::Path;

static DATABASE_SETUP_LOCK: Mutex<()> = Mutex::const_new(());
const TEST_POOL_MAX_CONNECTIONS: u32 = 4;

/// Isolated test environment with a fresh migrated database.
///
/// Each `TestContext` creates its own unique database on the PostgreSQL
/// container owned by the current test session. `make backend-test` starts one
/// pgvector container, PgBouncer and one NATS container before nextest, then
/// removes them after the session.
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
    _postgres: PostgresContainer,
    pgbouncer: Option<PgbouncerContainer>,
    nats_container: Mutex<Option<NatsContainer>>,
    vault: TestVault,
    database_handle: hermes_test_session::TestDatabase,
}

impl TestContext {
    /// Create a new isolated test environment.
    ///
    /// 1. Reuses the pgvector container for this test session
    /// 2. Creates a unique database (uuid-based name)
    /// 3. Reuses the session PgBouncer endpoint when a session is active
    /// 4. Runs migrations through that endpoint (or directly for standalone tests)
    /// 5. Returns a ready-to-use pool
    pub async fn new() -> Self {
        let container = PostgresContainer::start().await;
        let db_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', "_"));

        let _setup_guard = DATABASE_SETUP_LOCK.lock().await;
        let pgbouncer = PgbouncerContainer::start_for_current_session().await;
        let pool = create_database(&container, pgbouncer.as_ref(), &db_name).await;
        let vault = new_test_vault();
        let connection_string = match pgbouncer.as_ref() {
            Some(pgbouncer) => pgbouncer.connection_string(&db_name),
            None => direct_connection_string(container.host_port(), &db_name),
        };

        Self {
            _postgres: container,
            pgbouncer,
            nats_container: Mutex::new(None),
            vault,
            database_handle: hermes_test_session::TestDatabase::new(pool, connection_string),
        }
    }

    /// The connection pool to the isolated test database.
    pub fn pool(&self) -> &PgPool {
        self.database_handle.pool()
    }

    /// The full connection string for this test database.
    pub fn connection_string(&self) -> String {
        match &self.pgbouncer {
            _ => self.database_handle.connection_string().to_owned(),
        }
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
        let mut container = self.nats_container.lock().await;
        if container.is_none() {
            *container = Some(NatsContainer::start().await);
        }
        container
            .as_ref()
            .expect("NATS test container should be initialized")
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
        Database::from_test_pool(self.database_handle.clone_pool(), self.connection_string())
    }
}

async fn create_database(
    container: &PostgresContainer,
    pgbouncer: Option<&PgbouncerContainer>,
    db_name: &str,
) -> PgPool {
    let admin_pool = connect_with_retry(&container.connection_string(), "admin database").await;
    let create_sql = format!("CREATE DATABASE \"{}\"", db_name.replace('"', "\"\""));
    sqlx::query(&create_sql)
        .execute(&admin_pool)
        .await
        .unwrap_or_else(|error| panic!("failed to create database '{db_name}': {error}"));
    admin_pool.close().await;

    let database_url = match pgbouncer {
        Some(pgbouncer) => pgbouncer.connection_string(db_name),
        None => direct_connection_string(container.host_port(), db_name),
    };
    let pool = connect_with_retry(&database_url, "new test database").await;
    hermes_test_session::run_migrations(&pool)
        .await
        .expect("failed to run migrations");
    hermes_hub_backend::platform::settings::store::ApplicationSettingsStore::new(pool.clone())
        .repair_declared_settings()
        .await
        .expect("failed to repair application settings");
    pool
}

fn direct_connection_string(host_port: u16, database_name: &str) -> String {
    format!("postgres://testuser:testpass@127.0.0.1:{host_port}/{database_name}")
}

async fn connect_with_retry(database_url: &str, label: &str) -> PgPool {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        match PgPoolOptions::new()
            .max_connections(TEST_POOL_MAX_CONNECTIONS)
            .connect(database_url)
            .await
        {
            Ok(pool) => return pool,
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
            Err(error) => panic!("failed to connect to {label}: {error}"),
        }
    }
}
