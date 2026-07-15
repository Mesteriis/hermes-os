use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPortal {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub url: String,
    pub portal_type: String,
    pub login_hint: Option<String>,
    pub secret_reference: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgPortalStore {
    pool: PgPool,
}

impl OrgPortalStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgPortal>, OrgWorkflowError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, name, url, portal_type, login_hint,
                   secret_reference, last_used_at, notes, created_at
            FROM organization_portals
            WHERE organization_id=$1
            ORDER BY portal_type, name
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(portal_from_row).collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        url: &str,
        portal_type: &str,
    ) -> Result<OrgPortal, OrgWorkflowError> {
        let row = sqlx::query(
            r#"
            INSERT INTO organization_portals (organization_id, name, url, portal_type)
            VALUES ($1,$2,$3,$4)
            RETURNING id::text, organization_id, name, url, portal_type, login_hint,
                      secret_reference, last_used_at, notes, created_at
            "#,
        )
        .bind(org_id)
        .bind(name)
        .bind(url)
        .bind(portal_type)
        .fetch_one(&self.pool)
        .await?;

        portal_from_row(row)
    }
}

fn portal_from_row(row: sqlx::postgres::PgRow) -> Result<OrgPortal, OrgWorkflowError> {
    Ok(OrgPortal {
        id: row.try_get("id")?,
        organization_id: row.try_get("organization_id")?,
        name: row.try_get("name")?,
        url: row.try_get("url")?,
        portal_type: row.try_get("portal_type")?,
        login_hint: row.try_get("login_hint")?,
        secret_reference: row.try_get("secret_reference")?,
        last_used_at: row.try_get("last_used_at")?,
        notes: row.try_get("notes")?,
        created_at: row.try_get("created_at")?,
    })
}
