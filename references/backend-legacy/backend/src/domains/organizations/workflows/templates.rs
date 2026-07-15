use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgTemplate {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub template_type: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub language: Option<String>,
    pub tone: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgTemplateStore {
    pool: PgPool,
}

impl OrgTemplateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgTemplate>, OrgWorkflowError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, name, template_type, subject, body, language,
                   tone, metadata, created_at, updated_at
            FROM organization_templates
            WHERE organization_id=$1
            ORDER BY name
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgTemplate {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    template_type: row.try_get("template_type")?,
                    subject: row.try_get("subject")?,
                    body: row.try_get("body")?,
                    language: row.try_get("language")?,
                    tone: row.try_get("tone")?,
                    metadata: row.try_get("metadata")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}
