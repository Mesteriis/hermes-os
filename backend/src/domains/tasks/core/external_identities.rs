use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::TaskCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalTaskIdentity {
    pub id: String,
    pub task_id: String,
    pub provider: String,
    pub account_id: Option<String>,
    pub external_project_id: Option<String>,
    pub external_task_id: Option<String>,
    pub external_url: Option<String>,
    pub external_status: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ExternalTaskIdentityStore {
    pool: PgPool,
}

impl ExternalTaskIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, task_id: &str) -> Result<Vec<ExternalTaskIdentity>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, task_id, provider, account_id, external_project_id,
                   external_task_id, external_url, external_status, sync_status,
                   last_synced_at, created_at, updated_at
            FROM external_task_identities
            WHERE task_id = $1
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ExternalTaskIdentity {
                    id: row.try_get("id")?,
                    task_id: row.try_get("task_id")?,
                    provider: row.try_get("provider")?,
                    account_id: row.try_get("account_id")?,
                    external_project_id: row.try_get("external_project_id")?,
                    external_task_id: row.try_get("external_task_id")?,
                    external_url: row.try_get("external_url")?,
                    external_status: row.try_get("external_status")?,
                    sync_status: row.try_get("sync_status")?,
                    last_synced_at: row.try_get("last_synced_at")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}
