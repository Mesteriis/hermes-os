use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::OrgCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationIdentity {
    pub id: String,
    pub organization_id: String,
    pub identity_type: String,
    pub identity_value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub status: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgIdentityStore {
    pool: PgPool,
}

impl OrgIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrganizationIdentity>, OrgCoreError> {
        let rows = sqlx::query("SELECT id::text, organization_id, identity_type, identity_value, source, confidence, last_verified_at, status, metadata, created_at, updated_at FROM organization_identities WHERE organization_id = $1 ORDER BY identity_type")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrganizationIdentity {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    identity_type: row.try_get("identity_type")?,
                    identity_value: row.try_get("identity_value")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    last_verified_at: row.try_get("last_verified_at")?,
                    status: row.try_get("status")?,
                    metadata: row.try_get("metadata")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn upsert(
        &self,
        org_id: &str,
        itype: &str,
        ivalue: &str,
        source: &str,
    ) -> Result<OrganizationIdentity, OrgCoreError> {
        let row = sqlx::query("INSERT INTO organization_identities (organization_id, identity_type, identity_value, source) VALUES ($1,$2,$3,$4) ON CONFLICT (identity_type, identity_value) WHERE status='active' DO UPDATE SET updated_at=now() RETURNING id::text, organization_id, identity_type, identity_value, source, confidence, last_verified_at, status, metadata, created_at, updated_at")
            .bind(org_id)
            .bind(itype)
            .bind(ivalue)
            .bind(source)
            .fetch_one(&self.pool)
            .await?;

        Ok(OrganizationIdentity {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            identity_type: row.try_get("identity_type")?,
            identity_value: row.try_get("identity_value")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            last_verified_at: row.try_get("last_verified_at")?,
            status: row.try_get("status")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
