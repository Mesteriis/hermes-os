use sqlx::postgres::PgPool;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::time::{Duration, Instant, sleep};

const POSTGRES_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const POSTGRES_CONNECT_RETRY_DELAY: Duration = Duration::from_millis(250);

pub struct PostgresContainer {
    _container: ContainerAsync<GenericImage>,
    host_port: u16,
}

impl PostgresContainer {
    pub async fn start() -> Self {
        // GenericImage methods (return Self) must come BEFORE ImageExt methods (return ContainerRequest)
        let container = GenericImage::new("pgvector/pgvector", "0.8.2-pg16")
            .with_wait_for(WaitFor::message_on_stdout(
                "database system is ready to accept connections",
            ))
            .with_exposed_port(5432.tcp())
            // ImageExt methods return ContainerRequest<GenericImage>
            .with_env_var("POSTGRES_DB", "testdb")
            .with_env_var("POSTGRES_USER", "testuser")
            .with_env_var("POSTGRES_PASSWORD", "testpass")
            .start()
            .await
            .expect("failed to start pgvector container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to resolve pgvector container port");

        Self {
            _container: container,
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

        pool
    }

    pub fn host_port(&self) -> u16 {
        self.host_port
    }
}

async fn connect_with_retry(database_url: &str, label: &str) -> PgPool {
    let deadline = Instant::now() + POSTGRES_CONNECT_TIMEOUT;
    loop {
        match PgPool::connect(database_url).await {
            Ok(pool) => return pool,
            Err(_error) if Instant::now() < deadline => {
                sleep(POSTGRES_CONNECT_RETRY_DELAY).await;
            }
            Err(error) => panic!("failed to connect to {label}: {error}"),
        }
    }
}
