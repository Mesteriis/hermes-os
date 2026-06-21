use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPlaybook {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub trigger_condition: Option<String>,
    pub steps: Value,
    pub approval_mode: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgPlaybookStore {
    pool: PgPool,
}

impl OrgPlaybookStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgPlaybook>, OrgWorkflowError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, name, trigger_condition, steps, approval_mode,
                   enabled, last_run_at, created_at, updated_at
            FROM organization_playbooks
            WHERE organization_id=$1
            ORDER BY name
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgPlaybook {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    trigger_condition: row.try_get("trigger_condition")?,
                    steps: row.try_get("steps")?,
                    approval_mode: row.try_get("approval_mode")?,
                    enabled: row.try_get("enabled")?,
                    last_run_at: row.try_get("last_run_at")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}
