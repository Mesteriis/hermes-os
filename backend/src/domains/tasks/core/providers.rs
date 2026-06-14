use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::TaskCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskProviderAccount {
    pub account_id: String,
    pub provider: String,
    pub account_name: String,
    pub credentials_reference: Option<String>,
    pub sync_mode: String,
    pub capabilities: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskProviderStore {
    pool: PgPool,
}

impl TaskProviderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<TaskProviderAccount>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT account_id, provider, account_name, credentials_reference,
                   sync_mode, capabilities, created_at, updated_at
            FROM task_provider_accounts
            ORDER BY provider, account_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskProviderAccount {
                    account_id: row.try_get("account_id")?,
                    provider: row.try_get("provider")?,
                    account_name: row.try_get("account_name")?,
                    credentials_reference: row.try_get("credentials_reference")?,
                    sync_mode: row.try_get("sync_mode")?,
                    capabilities: row.try_get("capabilities")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        provider: &str,
        account_name: &str,
    ) -> Result<TaskProviderAccount, TaskCoreError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let account_id = format!("tprov:v1:{ts:x}");
        let row = sqlx::query(
            r#"
            INSERT INTO task_provider_accounts (account_id, provider, account_name)
            VALUES ($1, $2, $3)
            RETURNING account_id, provider, account_name, credentials_reference,
                      sync_mode, capabilities, created_at, updated_at
            "#,
        )
        .bind(&account_id)
        .bind(provider)
        .bind(account_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskProviderAccount {
            account_id: row.try_get("account_id")?,
            provider: row.try_get("provider")?,
            account_name: row.try_get("account_name")?,
            credentials_reference: row.try_get("credentials_reference")?,
            sync_mode: row.try_get("sync_mode")?,
            capabilities: row.try_get("capabilities")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
