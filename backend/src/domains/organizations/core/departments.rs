use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;

use super::errors::OrgCoreError;
use super::evidence::link_entity_in_transaction;

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
        let mut transaction = self.pool.begin().await?;
        let department =
            Self::add_in_transaction(&mut transaction, org_id, name, description, parent_id)
                .await?;
        transaction.commit().await?;
        Ok(department)
    }

    pub async fn add_with_observation(
        &self,
        org_id: &str,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
        observation_id: &str,
    ) -> Result<OrgDepartment, OrgCoreError> {
        let mut transaction = self.pool.begin().await?;
        let department =
            Self::add_in_transaction(&mut transaction, org_id, name, description, parent_id)
                .await?;
        link_entity_in_transaction(
            &mut transaction,
            observation_id,
            "department",
            &department.id,
            json!({
                "organization_id": org_id,
                "name": department.name,
            }),
        )
        .await?;
        transaction.commit().await?;
        Ok(department)
    }

    pub(crate) async fn add_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
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
            .fetch_one(&mut **transaction)
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
