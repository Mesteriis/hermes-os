use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelatedOrganization {
    pub id: String,
    pub organization_id: String,
    pub related_organization_id: String,
    pub relation_type: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RelatedOrgStore {
    pool: PgPool,
}

impl RelatedOrgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<RelatedOrganization>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, related_organization_id, relation_type, source, confidence::float8 AS confidence, created_at FROM related_organizations WHERE organization_id=$1")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(RelatedOrganization {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    related_organization_id: row.try_get("related_organization_id")?,
                    relation_type: row.try_get("relation_type")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn relate(
        &self,
        org_id: &str,
        related_id: &str,
        rel_type: &str,
    ) -> Result<RelatedOrganization, OrgCoreError> {
        let row = sqlx::query("INSERT INTO related_organizations (organization_id, related_organization_id, relation_type) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING RETURNING id::text, organization_id, related_organization_id, relation_type, source, confidence::float8 AS confidence, created_at")
            .bind(org_id)
            .bind(related_id)
            .bind(rel_type)
            .fetch_one(&self.pool)
            .await?;

        Ok(RelatedOrganization {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            related_organization_id: row.try_get("related_organization_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
