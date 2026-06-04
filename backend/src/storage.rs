use serde::Serialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use thiserror::Error;

use crate::event_log::{EventStoreError, run_migrations};

#[derive(Clone)]
pub struct Database {
    pool: Option<PgPool>,
}

impl Database {
    pub async fn connect(database_url: Option<&str>) -> Result<Self, StorageError> {
        let Some(database_url) = database_url else {
            return Ok(Self::disabled());
        };

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        run_migrations(&pool).await?;

        Ok(Self { pool: Some(pool) })
    }

    pub fn disabled() -> Self {
        Self { pool: None }
    }

    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    pub async fn readiness(&self) -> DatabaseReadiness {
        let Some(pool) = &self.pool else {
            return DatabaseReadiness::not_configured();
        };

        match sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
        {
            Ok(1) => DatabaseReadiness::ok(),
            Ok(_) => DatabaseReadiness::unavailable(
                "database readiness query returned unexpected result",
            ),
            Err(error) => {
                tracing::warn!(error = %error, "database readiness check failed");
                DatabaseReadiness::unavailable("database readiness query failed")
            }
        }
    }

    pub async fn migration_readiness(&self) -> MigrationReadiness {
        let Some(pool) = &self.pool else {
            return MigrationReadiness::not_configured();
        };

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM (
                VALUES
                    ('event_log'),
                    ('projection_cursors'),
                    ('api_audit_log')
            ) AS required_tables(table_name)
            WHERE to_regclass(required_tables.table_name) IS NOT NULL
            "#,
        )
        .fetch_one(pool)
        .await;

        match result {
            Ok(3) => MigrationReadiness::ok(),
            Ok(_) => MigrationReadiness::unavailable("required database schema is incomplete"),
            Err(error) => {
                tracing::warn!(error = %error, "database migration readiness check failed");
                MigrationReadiness::unavailable("database migration readiness query failed")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DatabaseReadiness {
    status: ReadinessStatus,
    message: &'static str,
}

impl DatabaseReadiness {
    fn ok() -> Self {
        Self {
            status: ReadinessStatus::Ok,
            message: "database is reachable",
        }
    }

    fn not_configured() -> Self {
        Self {
            status: ReadinessStatus::NotConfigured,
            message: "DATABASE_URL is not configured",
        }
    }

    fn unavailable(message: &'static str) -> Self {
        Self {
            status: ReadinessStatus::Unavailable,
            message,
        }
    }

    pub fn status(&self) -> ReadinessStatus {
        self.status
    }

    pub fn message(&self) -> &str {
        self.message
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MigrationReadiness {
    status: ReadinessStatus,
    message: &'static str,
}

impl MigrationReadiness {
    fn ok() -> Self {
        Self {
            status: ReadinessStatus::Ok,
            message: "required database schema is present",
        }
    }

    fn not_configured() -> Self {
        Self {
            status: ReadinessStatus::NotConfigured,
            message: "DATABASE_URL is not configured",
        }
    }

    fn unavailable(message: &'static str) -> Self {
        Self {
            status: ReadinessStatus::Unavailable,
            message,
        }
    }

    pub fn status(&self) -> ReadinessStatus {
        self.status
    }

    pub fn message(&self) -> &str {
        self.message
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessStatus {
    Ok,
    NotConfigured,
    Unavailable,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to connect to PostgreSQL")]
    Connect(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),
}
