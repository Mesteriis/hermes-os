use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgProcedure {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Value,
    pub source: String,
    pub confidence: f64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgProcedureStore {
    pool: PgPool,
}

impl OrgProcedureStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgProcedure>, OrgWorkflowError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, name, description, steps, source,
                   confidence::float8 AS confidence,
                   last_used_at, created_at, updated_at
            FROM organization_procedures
            WHERE organization_id=$1
            ORDER BY name
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgProcedure {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    steps: row.try_get("steps")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    last_used_at: row.try_get("last_used_at")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}
