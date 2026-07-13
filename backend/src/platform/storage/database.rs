use serde::Serialize;
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::platform::events::migrations::expected_migration_summary;
use crate::platform::events::migrations::run_migrations;
use crate::platform::settings::{ApplicationSettingsStore, SettingsError};
use crate::platform::storage::errors::StorageError;
use crate::platform::storage::models::{DatabaseReadiness, MigrationReadiness};
use hermes_events_postgres::errors::EventStoreError;

const DEFAULT_DATABASE_MAX_CONNECTIONS: u32 = 32;

#[derive(Clone)]
pub struct Database {
    pool: Option<PgPool>,
    database_url: Option<String>,
}

impl Database {
    pub async fn connect(database_url: Option<&str>) -> Result<Self, StorageError> {
        let Some(database_url) = database_url else {
            return Ok(Self::disabled());
        };

        let pool = PgPoolOptions::new()
            .max_connections(DEFAULT_DATABASE_MAX_CONNECTIONS)
            .connect(database_url)
            .await?;
        run_migrations(&pool).await?;
        let settings_repair = ApplicationSettingsStore::new(pool.clone())
            .repair_declared_settings()
            .await?;
        if settings_repair.changed() {
            tracing::warn!(
                inserted = settings_repair.inserted,
                repaired = settings_repair.repaired,
                reset_values = settings_repair.reset_values,
                "application settings were repaired during database startup"
            );
        }

        Ok(Self {
            pool: Some(pool),
            database_url: Some(database_url.to_owned()),
        })
    }

    pub fn disabled() -> Self {
        Self {
            pool: None,
            database_url: None,
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn from_test_pool(pool: PgPool, database_url: impl Into<String>) -> Self {
        Self {
            pool: Some(pool),
            database_url: Some(database_url.into()),
        }
    }

    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    pub(crate) fn database_url(&self) -> Option<&str> {
        self.database_url.as_deref()
    }

    pub async fn size_bytes(&self) -> Result<Option<u64>, StorageError> {
        let Some(pool) = &self.pool else {
            return Ok(None);
        };

        let size = sqlx::query_scalar::<_, i64>("SELECT pg_database_size(current_database())")
            .fetch_one(pool)
            .await?;

        Ok(u64::try_from(size).ok())
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

        let expected = expected_migration_summary();
        let result = sqlx::query_as::<_, AppliedMigrationSummary>(
            r#"
            SELECT
                count(*) FILTER (WHERE success) AS applied_count,
                COALESCE(max(version) FILTER (WHERE success), 0) AS latest_version,
                count(*) FILTER (WHERE NOT success) AS failed_count
            FROM _sqlx_migrations
            "#,
        )
        .fetch_one(pool)
        .await;

        match result {
            Ok(summary) if summary.matches(expected) => MigrationReadiness::ok(),
            Ok(summary) if summary.failed_count > 0 => {
                MigrationReadiness::unavailable("database migrations contain failed entries")
            }
            Ok(_) => MigrationReadiness::unavailable("required database migrations are incomplete"),
            Err(error) => {
                tracing::warn!(error = %error, "database migration readiness check failed");
                MigrationReadiness::unavailable("database migration readiness query failed")
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct AppliedMigrationSummary {
    applied_count: i64,
    latest_version: i64,
    failed_count: i64,
}

impl AppliedMigrationSummary {
    fn matches(&self, expected: crate::platform::events::migrations::MigrationSummary) -> bool {
        self.failed_count == 0
            && self.applied_count == expected.count
            && self.latest_version == expected.latest_version
    }
}
