use sqlx::postgres::{PgPool, PgPoolOptions};
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::time::{Duration, Instant, sleep};

use crate::containers::labels::{session_id_label_value, testkit_labels};

const POSTGRES_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const POSTGRES_CONNECT_RETRY_DELAY: Duration = Duration::from_millis(250);
const TEST_POOL_MAX_CONNECTIONS: u32 = 4;
const TEST_POSTGRES_MAX_CONNECTIONS: &str = "300";
pub const SESSION_POSTGRES_HOST_PORT_ENV: &str = "HERMES_TEST_POSTGRES_HOST_PORT";

pub struct PostgresContainer {
    _container: Option<ContainerAsync<GenericImage>>,
    host_port: u16,
}

impl PostgresContainer {
    pub async fn start() -> Self {
        if let Some(host_port) = session_host_port() {
            return Self {
                _container: None,
                host_port,
            };
        }

        Self::start_owned().await
    }

    pub async fn start_owned() -> Self {
        Self::start_owned_with_session(&session_id_label_value()).await
    }

    pub async fn start_owned_with_session(session_id: &str) -> Self {
        // GenericImage methods (return Self) must come BEFORE ImageExt methods (return ContainerRequest)
        let container = GenericImage::new("pgvector/pgvector", "0.8.2-pg16")
            .with_wait_for(WaitFor::message_on_stdout(
                "database system is ready to accept connections",
            ))
            .with_exposed_port(5432.tcp())
            // ImageExt methods return ContainerRequest<GenericImage>
            .with_labels(testkit_labels("postgres", session_id))
            .with_env_var("POSTGRES_DB", "testdb")
            .with_env_var("POSTGRES_USER", "testuser")
            .with_env_var("POSTGRES_PASSWORD", "testpass")
            .with_cmd(vec![
                "postgres".to_owned(),
                "-c".to_owned(),
                format!("max_connections={TEST_POSTGRES_MAX_CONNECTIONS}"),
            ])
            .start()
            .await
            .expect("failed to start pgvector container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to resolve pgvector container port");

        Self {
            _container: Some(container),
            host_port,
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://testuser:testpass@127.0.0.1:{}/testdb",
            self.host_port
        )
    }

    pub async fn create_database(&self, db_name: &str) -> PgPool {
        let admin_url = format!(
            "postgres://testuser:testpass@127.0.0.1:{}/testdb",
            self.host_port
        );

        let admin_pool = connect_with_retry(&admin_url, "admin database").await;

        let create_sql = format!("CREATE DATABASE \"{}\"", db_name.replace('"', "\"\""));
        sqlx::query(&create_sql)
            .execute(&admin_pool)
            .await
            .unwrap_or_else(|e| panic!("failed to create database '{db_name}': {e}"));

        admin_pool.close().await;

        let db_url = format!(
            "postgres://testuser:testpass@127.0.0.1:{}/{}",
            self.host_port, db_name
        );

        let pool = connect_with_retry(&db_url, "new test database").await;

        hermes_hub_backend::platform::events::run_migrations(&pool)
            .await
            .expect("failed to run migrations");
        hermes_hub_backend::platform::settings::ApplicationSettingsStore::new(pool.clone())
            .repair_declared_settings()
            .await
            .expect("failed to repair application settings");

        pool
    }

    pub fn host_port(&self) -> u16 {
        self.host_port
    }
}

fn session_host_port() -> Option<u16> {
    let value = std::env::var(SESSION_POSTGRES_HOST_PORT_ENV).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.parse::<u16>().unwrap_or_else(|error| {
        panic!("invalid {SESSION_POSTGRES_HOST_PORT_ENV} value '{trimmed}': {error}")
    }))
}

async fn connect_with_retry(database_url: &str, label: &str) -> PgPool {
    let deadline = Instant::now() + POSTGRES_CONNECT_TIMEOUT;
    loop {
        match PgPoolOptions::new()
            .max_connections(TEST_POOL_MAX_CONNECTIONS)
            .connect(database_url)
            .await
        {
            Ok(pool) => return pool,
            Err(_error) if Instant::now() < deadline => {
                sleep(POSTGRES_CONNECT_RETRY_DELAY).await;
            }
            Err(error) => panic!("failed to connect to {label}: {error}"),
        }
    }
}
