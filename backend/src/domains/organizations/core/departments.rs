use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::OrgCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgDepartment {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_department_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgDepartmentStore {
    pool: PgPool,
}

impl OrgDepartmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgDepartment>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, name, description, parent_department_id::text, created_at FROM organization_departments WHERE organization_id=$1 ORDER BY name")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgDepartment {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    parent_department_id: row.try_get("parent_department_id")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<OrgDepartment, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_departments (organization_id, name, description, parent_department_id) VALUES ($1,$2,$3,$4::uuid) RETURNING id::text, organization_id, name, description, parent_department_id::text, created_at")
            .bind(org_id)
            .bind(name)
            .bind(description)
            .bind(parent_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(OrgDepartment {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            parent_department_id: row.try_get("parent_department_id")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
